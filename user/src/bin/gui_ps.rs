// #![no_std]
// #![no_main]

// use alloc::format;
// use user_lib::{exec, fork, read_proc, sleep, wait4, TaskInfo, TaskStatus, WaitOption};
// use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};
// use embedded_graphics::pixelcolor::Rgb888;
// use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
// use embedded_graphics::prelude::{DrawTarget, Drawable, Point, Primitive, RgbColor, Size};
// use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
// use embedded_graphics::text::Text;

// #[macro_use]
// extern crate user_lib;
// extern crate alloc;

// use alloc::vec::Vec;

// // 进程状态对应的颜色
// const COLOR_READY: Rgb888 = Rgb888::new(0, 255, 0);    // 绿色
// const COLOR_RUNNING: Rgb888 = Rgb888::new(255, 0, 0);    // 红色
// const COLOR_BLOCKED: Rgb888 = Rgb888::new(255, 255, 0);  // 黄色
// const COLOR_DEAD: Rgb888 = Rgb888::new(128, 128, 128);  // 灰色

// struct ProcessVisualizer {
//     disp: Display,
//     block_size: u32,          // 每个进程方块的大小（像素）
//     time_slice_pixel: u32,    // 每个时间片（10ms）的像素宽度
//     text_y_offset: u32,       // 文本显示的垂直偏移量
// }

// impl ProcessVisualizer {
//     pub fn new() -> Self {
//         Self {
//             disp: Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES)),
//             block_size: 20,               // 方块大小20x20
//             time_slice_pixel: 20,         // 10ms对应20像素
//             text_y_offset: 5,             // 文本在方块下方5像素
//         }
//     }

//     fn get_process_color(&self, status: TaskStatus) -> Rgb888 {
//         match status {
//             TaskStatus::Ready => COLOR_READY,
//             TaskStatus::Running => COLOR_RUNNING,
//             TaskStatus::Blocked => COLOR_BLOCKED,
//             TaskStatus::Dead => COLOR_DEAD,
//         }
//     }

//     pub fn draw_processes(&mut self) {
//         // 清除屏幕
//         let _ = self.disp.clear(Rgb888::BLACK);

//         let mut processes = read_all();
//         // 按 last_time 排序
//         processes.sort_by_key(|p| p.first_time);

//         for (index, process) in processes.iter().enumerate() {
//             // 计算 y 坐标，每个进程占一行，间隔 block_size + 10 像素
//             let y = index as u32 * (self.block_size + 10);
//             // 计算 x 坐标，last_time 乘以每个时间片的像素宽度（10ms对应20像素，即1ms对应2像素）
//             let x = process.first_time as u32 * (self.time_slice_pixel / 10);

//             let color = self.get_process_color(process.status);
//             let style = PrimitiveStyle::with_fill(color);

//             // 绘制方块
//             let rect = Rectangle::new(
//                 Point::new(x as i32, y as i32),
//                 Size::new(self.block_size, self.block_size),
//             );
//             if let Err(e) = rect.into_styled(style).draw(&mut self.disp) {
//                 println!("Draw rectangle error: {:?}", e);
//             }

//             // 绘制 PID
//             let text = format!("PID: {}", process.pid);
//             let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
//             let text_point = Point::new(x as i32, y as i32 + self.block_size as i32 + self.text_y_offset as i32);
//             if let Err(e) = Text::new(&text, text_point, text_style).draw(&mut self.disp) {
//                 println!("Draw text error: {:?}", e);
//             }
//         }

//         // 刷新显示
//         self.disp.flush();
//     }
// }

// fn read_all() -> Vec<TaskInfo> {
//     let mut processes = Vec::new();
//     for pid in 0..512 {
//         let mut info = TaskInfo::new();
//         if read_proc(pid, &mut info) == 0 && info.status != TaskStatus::Dead {
//             processes.push(info);
//         }
//     }
//     processes
// }

// #[no_mangle]
// pub fn main(argc: usize, argv: &[&str]) -> isize {
//     if argc != 2 {
//         println!("Usage: gui_ps <app_name>");
//         return 0;
//     }

//     let pid = fork();
//     if pid == 0 {
//         exec(argv[1], &[core::ptr::null::<u8>()]);
//     } else {
//         let mut visualizer = ProcessVisualizer::new();
//         let mut exit_code: i32 = pid as i32 + 1;

//         println!("Child PID: {}", pid);
//         loop {
//             wait4(pid, &mut exit_code, WaitOption::WNOHANG.bits());
//             if exit_code == 0 {
//                 break;
//             }

//             visualizer.draw_processes();
//             sleep(100); // 控制刷新频率（毫秒）
//         }
//     }
//     0
// }


#![no_std]
#![no_main]

use core::convert::TryInto;

use alloc::format;
use user_lib::{exec, fork, read_proc, sleep, wait4, TaskInfo, TaskStatus, WaitOption};
use user_lib::{Display, VIRTGPU_XRES, VIRTGPU_YRES};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, Line};
use embedded_graphics::prelude::{DrawTarget, Drawable, Point, Primitive, RgbColor, Size};
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;

#[macro_use]
extern crate user_lib;
extern crate alloc;

use alloc::vec::Vec;

// 进程状态对应的颜色
const COLOR_READY: Rgb888 = Rgb888::new(0, 255, 0);    // 绿色
const COLOR_RUNNING: Rgb888 = Rgb888::new(255, 0, 0);    // 红色
const COLOR_BLOCKED: Rgb888 = Rgb888::new(255, 255, 0);  // 黄色
const COLOR_DEAD: Rgb888 = Rgb888::new(128, 128, 128);  // 灰色

// 图形参数
const TIME_AXIS_HEIGHT: u32 = 40;       // 时间轴区域高度
const PROCESS_ROW_SPACING: u32 = 50;     // 进程行间距（像素）
const SCALE_TEXT_Y_OFFSET: u32 = 20;      // 刻度文本垂直偏移

struct ProcessVisualizer {
    disp: Display,
    block_size: u32,          // 进程块大小（像素）
    time_scale_pixel: u32,    // 每个时间单位（10ms）像素宽度
    base_x: u32,              // x轴原点（左边界）
    base_y: u32,              // y轴原点（底部）
}

impl ProcessVisualizer {
    pub fn new() -> Self {
        Self {
            disp: Display::new(Size::new(VIRTGPU_XRES, VIRTGPU_YRES)),
            block_size: 50,               // 进程块20x20像素
            time_scale_pixel: 50,         // 10ms对应20像素
            base_x: 30,                   // 左边距20像素
            base_y: VIRTGPU_YRES - TIME_AXIS_HEIGHT, // 底部原点
        }
    }

    fn get_process_color(&self, status: TaskStatus) -> Rgb888 {
        match status {
            TaskStatus::Ready => COLOR_READY,
            TaskStatus::Running => COLOR_RUNNING,
            TaskStatus::Blocked => COLOR_BLOCKED,
            TaskStatus::Dead => COLOR_DEAD,
        }
    }

    pub fn draw_processes(&mut self) {
        let mut processes = read_all();
        if processes.is_empty() {
            self.disp.flush();
            return;
        }

        // 数据准备
        processes.sort_by_key(|p| p.first_time);
        let (min_time, max_time) = self.calculate_time_range(&processes);
        let time_span = max_time - min_time;
        let available_width = VIRTGPU_XRES - self.base_x * 2;
        let optimal_scale = self.calculate_optimal_scale(time_span, available_width);

        // 清除屏幕并绘制背景
        self.disp.clear(Rgb888::BLACK).ok();
        self.draw_time_axis(min_time, max_time, optimal_scale);
        self.draw_process_blocks(&processes, min_time, optimal_scale);
        self.disp.flush();
    }

    fn calculate_time_range(&self, processes: &[TaskInfo]) -> (u32, u32) {
        let mut min_time = processes[0].first_time;
        let mut max_time = processes[0].first_time;
        for p in processes {
            if p.first_time < min_time { min_time = p.first_time; }
            if p.first_time > max_time { max_time = p.first_time; }
        }
        (min_time.try_into().unwrap(), max_time.try_into().unwrap())
    }

    fn calculate_optimal_scale(&self, time_span: u32, available_width: u32) -> u32 {
        // 增大刻度间隔（每个刻度占80像素），减少刻度数量
        let min_scale = 10; // 最小时间间隔（10ms）
        let max_scales = available_width / 80; 
        if time_span == 0 { return min_scale; }
        (time_span / max_scales).max(min_scale)
    }

    fn draw_time_axis(&mut self, min_time: u32, max_time: u32, scale: u32) {
        let style = PrimitiveStyle::with_stroke(Rgb888::WHITE, 2);
        
        // 绘制x轴（底部水平线）
        Line::new(
            Point::new(self.base_x as i32, self.base_y as i32),
            Point::new((VIRTGPU_XRES - self.base_x) as i32, self.base_y as i32)
        ).into_styled(style).draw(&mut self.disp).ok();

        // 绘制刻度和文本（增大刻度间隔）
        for time in (min_time..=max_time).step_by(scale as usize) {
            let x = self.base_x + ((time - min_time) * self.time_scale_pixel / 10) as u32;
            let scale_height = if time == min_time || time == max_time { 15 } else { 10 };
            
            // 刻度线
            Line::new(
                Point::new(x as i32, self.base_y as i32),
                Point::new(x as i32, self.base_y as i32 - scale_height as i32)
            ).into_styled(style).draw(&mut self.disp).ok();
            
            // 时间文本（底部居中，增大间距）
            let text = format!("{}ms", time);
            let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
            let text_width = text.len() as u32 * FONT_6X10.character_size.width;
            Text::new(
                &text,
                Point::new((x - text_width / 2) as i32, (self.base_y + SCALE_TEXT_Y_OFFSET).try_into().unwrap()),
                text_style
            ).draw(&mut self.disp).ok();
        }
    }

    fn draw_process_blocks(&mut self, processes: &[TaskInfo], min_time: u32, scale: u32) {
        for (row, process) in processes.iter().enumerate() {
            let y = self.base_y - (row as u32 + 1) * PROCESS_ROW_SPACING; // 从下往上排列
            
            // 计算x坐标（增大刻度对应像素）
            let x = self.base_x + (((process.first_time - min_time as usize) * self.time_scale_pixel as usize) / 10) as u32;
            
            // 绘制进程块
            Rectangle::new(
                Point::new(x as i32, y as i32),
                Size::new(self.block_size, self.block_size)
            ).into_styled(PrimitiveStyle::with_fill(self.get_process_color(process.status))).draw(&mut self.disp).ok();
            
            // 绘制PID（调整位置，增大垂直间距）
            let text = format!("{}", process.pid);
            let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
            Text::new(
                &text,
                Point::new(x as i32, y as i32 - self.block_size as i32 + 45), // 增大垂直间距
                text_style
            ).draw(&mut self.disp).ok();
        }
    }
}

fn read_all() -> Vec<TaskInfo> {
    let mut processes = Vec::new();
    for pid in 0..512 {
        let mut info = TaskInfo::new();
        if read_proc(pid, &mut info) == 0 && info.status != TaskStatus::Dead {
            processes.push(info);
        }
    }
    processes
}

#[no_mangle]
pub fn main(argc: usize, argv: &[&str]) -> isize {
    if argc != 2 {
        println!("Usage: gui_ps <app_name>");
        return 0;
    }

    let pid = fork();
    if pid == 0 {
        exec(argv[1], &[core::ptr::null::<u8>()]);
    } else {
        let mut visualizer = ProcessVisualizer::new();
        let mut exit_code: i32 = pid as i32 + 1;

        loop {
            wait4(pid, &mut exit_code, WaitOption::WNOHANG.bits());
            if exit_code == 0 {
                let _ = visualizer.disp.clear(Rgb888::BLACK);
                visualizer.disp.flush();
                break;
            }
            visualizer.draw_processes();
            sleep(5 * 100); // 优化刷新间隔
        }
    }
    0
}