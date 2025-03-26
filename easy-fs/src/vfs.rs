use super::{
    block_cache_sync_all, get_block_cache, BlockDevice, DirEntry, DiskInode, DiskInodeType,
    EasyFileSystem, DIRENT_SZ,
};
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};

pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
    inode_id: u32,
}

impl Inode {
    /// We should not acquire efs lock here.
    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
        inode_id: u32,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
            inode_id,
        }
    }
    pub fn get_inode_id(&self) -> u32 {
        self.inode_id
    }
    #[allow(unused)]
    fn self_dirent(&self) -> Arc<DirEntry> {
        let mut dirent = DirEntry::empty();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(0, dirent.as_bytes_mut(), &self.block_device));
        Arc::new(dirent)
    }

    #[allow(unused)]
    fn parent_dirent(&self) -> Arc<DirEntry> {
        let mut dirent = DirEntry::empty();
        self.read_at(DIRENT_SZ, dirent.as_bytes_mut());
        Arc::new(dirent)
    }

    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }

    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }

    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        // assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

    pub fn find(&self, path: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        let mut block_id = self.block_id as u32;
        let mut block_offset = self.block_offset;
        let mut new_inode_id = 0;
        // cut path and find inode. if inode is a file during the process, return None
        for name in path.split('/').filter(|s| !s.is_empty()) {
            let inode_id = get_block_cache(block_id as usize, self.block_device.clone())
                .lock()
                .read(block_offset, |disk_inode: &DiskInode| {
                    if disk_inode.is_file() {
                        return None;
                    }
                    self.find_inode_id(name, disk_inode)
                });
            if inode_id.is_none() {
                return None;
            }
            new_inode_id = inode_id.unwrap();
            // update block_id and block_offset
            (block_id, block_offset) = fs.get_disk_inode_pos(inode_id.unwrap());
        }
        // finally get it
        Some(Arc::new(Self::new(
            block_id, 
            block_offset, 
            self.fs.clone(), 
            self.block_device.clone(),
            new_inode_id,
        )))
    }

    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }

    /// Create regular file under current inode
    pub fn create_file(&self, name: &str) -> Option<Arc<Inode>> {
        self.create_inode(name, DiskInodeType::File)
    }

    /// Create directory under current inode
    pub fn create_dir(&self, name: &str) -> Option<Arc<Inode>> {
        self.create_inode(name, DiskInodeType::Directory)
    }

    // create inode under current inode
    fn create_inode(&self, name: &str, inode_type: DiskInodeType) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &mut DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir(), "node is not directory");
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.modify_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        let self_inode_id = self.self_dirent().inode_number();
        // insert inode into cache as required
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(inode_type);
                if inode_type == DiskInodeType::Directory {
                    // read '.' is read self
                    self.increase_size((DIRENT_SZ * 2) as u32, new_inode, &mut fs);
                    let dirent_parent = DirEntry::new("..", self_inode_id);
                    let dirent_self = DirEntry::new(".", new_inode_id);
                    new_inode.write_at(0, dirent_self.as_bytes(), &self.block_device);
                    new_inode.write_at(DIRENT_SZ, dirent_parent.as_bytes(), &self.block_device);
                }
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
            new_inode_id,
        )))
        // release efs lock automatically by compiler
    }

    // return nothing if it is a file
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let mut v: Vec<String> = Vec::new(); 
            if disk_inode.is_file() {
                return v;
            }
            let file_count = (disk_inode.size as usize) / DIRENT_SZ;
            for i in 0..file_count {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device,),
                    DIRENT_SZ,
                );
                v.push(String::from(dirent.name()));
            }
            v
        })
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| disk_inode.read_at(offset, buf, &self.block_device))
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let size = self.modify_disk_inode(|disk_inode| {
            // make sure you are writing a file
            // assert!(disk_inode.is_file());
            self.increase_size((offset + buf.len()) as u32, disk_inode, &mut fs);
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        size
    }

    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            assert!(disk_inode.is_file());
            let size = disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            assert!(data_blocks_dealloc.len() == DiskInode::total_blocks(size) as usize);
            for data_block in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(data_block);
            }
        });
        block_cache_sync_all();
    }
    /// Create a hard link with new_name for file with old_name
    pub fn linkat(&self, old_path: &str, new_path: &str) -> isize {
        // get inode of file with old_name
        let inode = self.find(old_path);
        if inode.is_none() {
            return -1;
        }
        let inode = inode.unwrap();
        inode.modify_disk_inode(|dinode| {
            dinode.nlink += 1;  // increase reference count
        });
        let new_ino = inode.get_inode_id();
        let mut fs = self.fs.lock();
        // insert a new directory entry into ROOT_DIR
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(new_path, new_ino as u32);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });
        0
    }
    /// Remove a hard link with name
    pub fn unlinkat(&self, path: &str) -> isize {
        // get inode of file with name
        let inode = self.find(path);
        if inode.is_none() {
            return -1;
        }
        let inode = inode.unwrap();

        let mut is_zero_link = false;
        inode.modify_disk_inode(|dinode| {
            dinode.nlink -= 1;
            if dinode.nlink == 0 {
                is_zero_link = true;
            }
        });

        let mut res = -1;
        self.modify_disk_inode(|dinode| {
            // Remove(For simplisity, not remove, just set to empty) 
            // the directory entry with name in the ROOT_DIR
            let fcnt = (dinode.size as usize) / DIRENT_SZ;
            for i in 0..fcnt {
                let mut dirent = DirEntry::empty();
                assert_eq!(
                    dinode.read_at(i * DIRENT_SZ, dirent.as_bytes_mut(), &self.block_device),
                    DIRENT_SZ
                );
                if dirent.name() == path {
                    dirent = DirEntry::empty();
                    dinode.write_at(i * DIRENT_SZ, dirent.as_bytes(), &self.block_device);
                    res = 0;
                    break;
                }
            }
        });
        res
    }
    /// Get nlink of the inode
    pub fn get_nlink(&self) -> u32 {
        let mut nlink = 0;
        self.read_disk_inode(|dinode| {
            nlink = dinode.nlink;
        });
        nlink
    }
    /// Get file type of the inode
    pub fn get_file_type(&self) -> DiskInodeType {
        let mut ftype = DiskInodeType::File;
        self.read_disk_inode(|dinode| {
            if dinode.is_dir() {
                ftype = DiskInodeType::Directory;
            }
        });
        ftype
    }
    /// Get inode number of the inode
    pub fn get_ino(&self) -> u32 {
        let fs = self.fs.lock();
        fs.get_ino(self.block_id, self.block_offset) as u32
    }
}
