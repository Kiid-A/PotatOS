# generate_large_file.py

# 定义文件名和大小
file_name = "large_file.bin"
file_size = 10 * 1024 * 1024  # 10MB

# 生成文件
with open(file_name, "wb") as f:
    f.write(b"\0" * file_size)  # 写入 10MB 的空字节

print(f"Generated {file_name} with size {file_size} bytes")
