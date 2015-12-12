use std::fs::File;
use std::io::Read;

use base::node;
use constants::SYSINFO_RATE;
use messages::SysInfo;


pub struct SysInformer {
    total_mem: u32,
    prev_idle: u32,
    prev_total: u32
}

impl SysInformer {
    fn get_loadavg(&self) -> (u8, u8, u8) {
        let content = self.read_file("/proc/loadavg");

        let mut split = content.split_whitespace();
        let one = split.next().unwrap().parse::<f32>().unwrap() * 100.;
        let five = split.next().unwrap().parse::<f32>().unwrap() * 100.;
        let fifteen = split.next().unwrap().parse::<f32>().unwrap() * 100.;

        (one as u8, five as u8, fifteen as u8)
    }

    fn get_mem(&mut self) -> (u8, u8) {
        let content = self.read_file("/proc/meminfo");

        let mut split = content.split_whitespace();

        if self.total_mem == 0 {
            self.total_mem = split.nth(1).unwrap().parse().unwrap();
        } else {
            split.nth(1);
        }

        let free: u32 = split.nth(2).unwrap().parse().unwrap();
        // Kernel 3.14+.
        let avail: u32 = split.nth(2).unwrap().parse().unwrap();

        ((free * 255 / self.total_mem) as u8, (avail * 255 / self.total_mem) as u8)
    }

    fn get_cpu(&mut self) -> u8 {
        let content = self.read_file("/proc/stat");

        let mut split = content.split_whitespace();
        let (mut total, mut idle) = (0u32, 0u32);

        total += split.nth(1).unwrap().parse::<u32>().unwrap()      // user
               + split.next().unwrap().parse::<u32>().unwrap()      // nice
               + split.next().unwrap().parse::<u32>().unwrap();     // system

        idle += split.next().unwrap().parse::<u32>().unwrap()       // idle
              + split.next().unwrap().parse::<u32>().unwrap();      // iowait

        total += split.next().unwrap().parse::<u32>().unwrap()      // irq
               + split.next().unwrap().parse::<u32>().unwrap()      // softirq
               + split.next().unwrap().parse::<u32>().unwrap()      // steal
               + idle;

        let d_idle = idle - self.prev_idle;
        let d_total = total - self.prev_total;

        self.prev_idle = idle;
        self.prev_total = total;

        ((d_total - d_idle) * 255 / d_total) as u8
    }

    fn get_temp(&self) -> i8 {
        let content = self.read_file("/sys/class/thermal/thermal_zone0/temp");

        (content.trim_right().parse::<f32>().unwrap() / 1000.).round() as i8
    }

    fn read_file(&self, path: &str) -> String {
        let mut file = File::open(path).unwrap();
        let mut string = String::new();
        file.read_to_string(&mut string).unwrap();
        string
    }
}

pub fn worker() {
    let sys_info_tx = node::advertise::<SysInfo>();

    let mut informer = SysInformer { total_mem: 0, prev_idle: 0, prev_total: 0 };

    info!("running at {}Hz", SYSINFO_RATE);

    for _ in node::periodic(SYSINFO_RATE) {
        let (free_mem, avail_mem) = informer.get_mem();
        let cpu = informer.get_cpu();
        let loadavg = informer.get_loadavg();
        let temp = informer.get_temp();

        sys_info_tx.send(SysInfo {
            free_mem: free_mem,
            avail_mem: avail_mem,
            cpu: cpu,
            loadavg: loadavg,
            temp: temp
        });
    }
}
