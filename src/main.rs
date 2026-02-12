#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, Stroke, Vec2, RichText, TextureHandle};
use sysinfo::{System, Disks, Networks};
use std::process::Command;
use std::os::windows::process::CommandExt;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicU8, Ordering}};
use winreg::enums::*;
use winreg::RegKey;
use wmi::{COMLibrary, WMIConnection};
use serde::Deserialize;

const CREATE_NO_WINDOW: u32 = 0x08000000;

fn ease_out_cubic(t: f32) -> f32 { 1.0 - (1.0 - t).powi(3) }
fn ease_out_back(t: f32) -> f32 { let c1 = 1.70158; let c3 = c1 + 1.0; 1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2) }

const ACCENT: Color32 = Color32::from_rgb(65, 145, 255);
const BG: Color32 = Color32::from_rgb(12, 12, 16);
const CARD: Color32 = Color32::from_rgb(22, 22, 28);
const CARD2: Color32 = Color32::from_rgb(32, 32, 40);
const TXT: Color32 = Color32::from_rgb(245, 245, 250);
const DIM: Color32 = Color32::from_rgb(130, 130, 145);
const CYAN: Color32 = Color32::from_rgb(100, 210, 255);
const GRN: Color32 = Color32::from_rgb(80, 220, 120);
const RED: Color32 = Color32::from_rgb(255, 90, 90);
const BORDER: Color32 = Color32::from_rgb(50, 50, 65);
const YLW: Color32 = Color32::from_rgb(255, 200, 60);

// SVG Icons (16x16)
const ICO_PC: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><rect x=\"1\" y=\"2\" width=\"14\" height=\"9\" rx=\"1\" fill=\"none\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"8\" y1=\"11\" x2=\"8\" y2=\"14\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"5\" y1=\"14\" x2=\"11\" y2=\"14\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";
const ICO_KEY: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><circle cx=\"5\" cy=\"11\" r=\"3\" fill=\"none\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"7\" y1=\"9\" x2=\"14\" y2=\"2\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"12\" y1=\"2\" x2=\"14\" y2=\"4\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"10\" y1=\"4\" x2=\"12\" y2=\"6\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";
const ICO_GEAR: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><circle cx=\"8\" cy=\"8\" r=\"2.5\" fill=\"none\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><path d=\"M8 1v2M8 13v2M1 8h2M13 8h2M2.9 2.9l1.4 1.4M11.7 11.7l1.4 1.4M2.9 13.1l1.4-1.4M11.7 4.3l1.4-1.4\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";
const ICO_ROCKET: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><path d=\"M8 1c-2 3-2 6-1 9l-3 2 1 2 3-1c1 1 2 1 3 0l3 1 1-2-3-2c1-3 1-6-1-9l-1.5 3-1.5-3z\" fill=\"none\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.2\"/></svg>";
const ICO_SHIELD: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><path d=\"M8 1L2 3v4c0 4 2.5 6.5 6 8 3.5-1.5 6-4 6-8V3L8 1z\" fill=\"none\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";
const ICO_DOWN: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><path d=\"M8 2v10M4 9l4 4 4-4\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\" fill=\"none\"/><line x1=\"3\" y1=\"14\" x2=\"13\" y2=\"14\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";
const ICO_MENU: &[u8] = b"<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 16 16\" fill=\"rgb(65,145,255)\"><line x1=\"2\" y1=\"4\" x2=\"14\" y2=\"4\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"2\" y1=\"8\" x2=\"14\" y2=\"8\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/><line x1=\"2\" y1=\"12\" x2=\"14\" y2=\"12\" stroke=\"rgb(65,145,255)\" stroke-width=\"1.5\"/></svg>";

fn main() -> eframe::Result<()> {
    eframe::run_native("Emporium Support", eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 500.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(false),
        ..Default::default()
    }, Box::new(|cc| { cc.egui_ctx.set_visuals(egui::Visuals::dark()); Ok(Box::new(App::new())) }))
}

#[derive(Default, PartialEq, Clone, Copy)]
enum Tab { #[default] System, Hwid, Windows, Boosters, AntiCheat, Installs, Controls }
impl Tab {
    fn name(&self) -> &'static str {
        match self { Tab::System=>"System", Tab::Hwid=>"HWID", Tab::Windows=>"Windows", Tab::Boosters=>"Boosters", Tab::AntiCheat=>"Anti-Cheat", Tab::Installs=>"Installs", Tab::Controls=>"Controls" }
    }
    fn all() -> &'static [Tab] { &[Tab::System, Tab::Hwid, Tab::Windows, Tab::Boosters, Tab::AntiCheat, Tab::Installs, Tab::Controls] }
}

#[derive(Default)]
struct Anim {
    tab: f32,
    tab_dir: i8,
    toast: f32,
    refresh_spin: f32,
    global_time: f32,
}

struct App {
    tab: Tab,
    sys: Sys, hwid: Hwid, win: Win, soft: Soft,
    msg: String, msg_t: f32,
    anim: Anim,
    install_state: [Arc<AtomicU8>; 5],
    icons: Option<Icons>,
    auto_refresh: bool,
    refresh_timer: f32,
    cmd_busy: Arc<AtomicBool>,
    needs_reload: Arc<AtomicBool>,
    pending_toast: Arc<Mutex<String>>,
    is_admin: bool,
}

struct Icons {
    pc: TextureHandle, key: TextureHandle, gear: TextureHandle,
    rocket: TextureHandle, shield: TextureHandle, down: TextureHandle,
    menu: TextureHandle,
}

#[derive(Default, Clone)] struct Sys { os: String, cpu: String, gpu: Vec<String>, ram: String, mb: String, drives: Vec<(String,String,String)>, macs: Vec<(String,String)> }
#[derive(Default, Clone)] struct Hwid { cpu: String, mb: String, bios: String, guid: String, pid: String, uuid: String, disks: Vec<(String,String)> }
#[derive(Default, Clone, Copy)] struct Win { uac: bool, def: bool, rt: bool, ss: bool, sb: bool }
#[derive(Default, Clone)] struct Soft { razer: bool, dragon: bool, ab: bool, vg: bool, faceit: bool, esea: bool, eac: bool, be: bool, mb: bool, vc13_64: bool, vc13_86: bool, vc22_64: bool, vc22_86: bool, dx: bool }

#[allow(non_camel_case_types, dead_code)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_Processor { name: Option<String>, processor_id: Option<String> }
#[allow(non_camel_case_types)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_VideoController { name: Option<String> }
#[allow(non_camel_case_types)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_BaseBoard { manufacturer: Option<String>, product: Option<String>, serial_number: Option<String> }
#[allow(non_camel_case_types)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_BIOS { serial_number: Option<String> }
#[allow(non_camel_case_types)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_DiskDrive { model: Option<String>, serial_number: Option<String> }
#[allow(non_camel_case_types)] #[derive(Deserialize, Debug)] #[serde(rename_all = "PascalCase")] struct Win32_ComputerSystemProduct { #[serde(rename = "UUID")] uuid: Option<String> }

fn check_admin() -> bool {
    Command::new("net").args(["session"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

impl App {
    fn new() -> Self {
        let is_admin = check_admin();
        let mut a = Self {
            tab: Tab::System, sys: Sys::default(), hwid: Hwid::default(), win: Win::default(),
            soft: Soft::default(), msg: String::new(), msg_t: 0.0,
            anim: Anim { tab: 1.0, ..Default::default() },
            install_state: std::array::from_fn(|_| Arc::new(AtomicU8::new(0))),
            icons: None,
            auto_refresh: false,
            refresh_timer: 30.0,
            cmd_busy: Arc::new(AtomicBool::new(false)),
            needs_reload: Arc::new(AtomicBool::new(false)),
            pending_toast: Arc::new(Mutex::new(String::new())),
            is_admin,
        };
        a.load();
        a
    }

    fn load_icons(&mut self, ctx: &egui::Context) {
        if self.icons.is_none() {
            let load = |data: &[u8], name: &str| -> Option<TextureHandle> {
                egui_extras::image::load_svg_bytes(data).ok()
                    .map(|img| ctx.load_texture(name, img, egui::TextureOptions::LINEAR))
            };
            if let (Some(pc), Some(key), Some(gear), Some(rocket), Some(shield), Some(down), Some(menu)) = (
                load(ICO_PC, "ico_pc"), load(ICO_KEY, "ico_key"), load(ICO_GEAR, "ico_gear"),
                load(ICO_ROCKET, "ico_rocket"), load(ICO_SHIELD, "ico_shield"), load(ICO_DOWN, "ico_down"),
                load(ICO_MENU, "ico_menu"),
            ) {
                self.icons = Some(Icons { pc, key, gear, rocket, shield, down, menu });
            }
        }
    }

    fn load(&mut self) {
        let mut s = System::new_all(); s.refresh_all();
        self.sys.cpu = s.cpus().first().map(|c| c.brand().to_string()).unwrap_or_default();
        self.sys.ram = format!("{} GB", s.total_memory()/1024/1024/1024);
        self.sys.os = System::long_os_version().unwrap_or_default();

        if let Ok(com) = COMLibrary::new() { if let Ok(wmi) = WMIConnection::new(com) {
            self.sys.gpu = wmi.query::<Win32_VideoController>().ok().map(|g| g.iter().filter_map(|x| x.name.clone()).collect()).unwrap_or_default();
            if let Ok(b) = wmi.query::<Win32_BaseBoard>() { if let Some(x) = b.first() { self.sys.mb = format!("{} {}", x.manufacturer.clone().unwrap_or_default(), x.product.clone().unwrap_or_default()); self.hwid.mb = x.serial_number.clone().unwrap_or_default(); } }
            if let Ok(c) = wmi.query::<Win32_Processor>() { if let Some(x) = c.first() { self.hwid.cpu = x.processor_id.clone().unwrap_or_default(); } }
            if let Ok(b) = wmi.query::<Win32_BIOS>() { if let Some(x) = b.first() { self.hwid.bios = x.serial_number.clone().unwrap_or_default(); } }
            if let Ok(d) = wmi.query::<Win32_DiskDrive>() { self.hwid.disks = d.iter().map(|x| (x.model.clone().unwrap_or_default(), x.serial_number.clone().unwrap_or_default().trim().to_string())).collect(); }
            if let Ok(p) = wmi.query::<Win32_ComputerSystemProduct>() { if let Some(x) = p.first() { self.hwid.uuid = x.uuid.clone().unwrap_or_default(); } }
        }}

        self.sys.drives = Disks::new_with_refreshed_list().iter()
            .filter(|d| d.total_space() > 1_000_000_000)
            .map(|d| (d.mount_point().to_string_lossy().to_string(), format!("{:.0}G", d.available_space() as f64/1e9), format!("{:.0}G", d.total_space() as f64/1e9)))
            .collect();

        self.sys.macs = Networks::new_with_refreshed_list().iter()
            .filter(|(n,d)| {
                let mac = d.mac_address().to_string();
                let name_lower = n.to_lowercase();
                mac != "00:00:00:00:00:00" &&
                !name_lower.contains("vethernet") &&
                !name_lower.contains("wsl") &&
                !name_lower.contains("hyper-v") &&
                !name_lower.contains("loopback") &&
                !name_lower.contains("virtual")
            })
            .map(|(n,d)| (n.clone(), d.mac_address().to_string()))
            .collect();

        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Cryptography") { self.hwid.guid = k.get_value("MachineGuid").unwrap_or_default(); }
        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") { self.hwid.pid = k.get_value("ProductId").unwrap_or_default(); }
        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System") { self.win.uac = k.get_value::<u32,_>("EnableLUA").unwrap_or(1) == 1; }
        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows Defender") { self.win.def = k.get_value::<u32,_>("DisableAntiSpyware").unwrap_or(0) == 0; }
        {
            let direct_off = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows Defender\Real-Time Protection")
                .ok().and_then(|k| k.get_value::<u32,_>("DisableRealtimeMonitoring").ok()).unwrap_or(0) == 1;
            let policy_off = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Policies\Microsoft\Windows Defender\Real-Time Protection")
                .ok().and_then(|k| k.get_value::<u32,_>("DisableRealtimeMonitoring").ok()).unwrap_or(0) == 1;
            self.win.rt = !direct_off && !policy_off;
        }
        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer") { self.win.ss = k.get_value::<String,_>("SmartScreenEnabled").unwrap_or_default() != "Off"; }
        if let Ok(o) = Command::new("powershell").args(["-Command", "Confirm-SecureBootUEFI"]).creation_flags(CREATE_NO_WINDOW).output() { self.win.sb = String::from_utf8_lossy(&o.stdout).trim() == "True"; }

        let p = progs(); let has = |s: &str| p.iter().any(|x| x.to_lowercase().contains(s));
        self.soft.razer = has("razer cortex"); self.soft.dragon = has("dragon center"); self.soft.ab = has("msi afterburner");
        self.soft.vg = has("vanguard"); self.soft.faceit = has("faceit"); self.soft.esea = has("esea"); self.soft.eac = has("easyanticheat"); self.soft.be = has("battleye"); self.soft.mb = has("malwarebytes");
        self.soft.vc13_64 = p.iter().any(|x| {
            let xl = x.to_lowercase();
            (xl.contains("visual c++") || xl.contains("vc++")) &&
            xl.contains("2013") && (xl.contains("x64") || xl.contains("64"))
        });
        self.soft.vc13_86 = p.iter().any(|x| {
            let xl = x.to_lowercase();
            (xl.contains("visual c++") || xl.contains("vc++")) &&
            xl.contains("2013") && (xl.contains("x86") || (xl.contains("32") && !xl.contains("64")))
        });
        self.soft.vc22_64 = p.iter().any(|x| {
            let xl = x.to_lowercase();
            (xl.contains("visual c++") || xl.contains("vc++")) &&
            (xl.contains("2015") || xl.contains("2017") || xl.contains("2019") || xl.contains("2022") || xl.contains("2015-2022")) &&
            (xl.contains("x64") || xl.contains("64-bit"))
        });
        self.soft.vc22_86 = p.iter().any(|x| {
            let xl = x.to_lowercase();
            (xl.contains("visual c++") || xl.contains("vc++")) &&
            (xl.contains("2015") || xl.contains("2017") || xl.contains("2019") || xl.contains("2022") || xl.contains("2015-2022")) &&
            (xl.contains("x86") || (xl.contains("32") && !xl.contains("64")))
        });
        self.soft.dx = std::path::Path::new(r"C:\Windows\System32\d3d11.dll").exists() &&
                       std::path::Path::new(r"C:\Windows\System32\dxgi.dll").exists();

        for (i, installed) in [self.soft.vc13_64, self.soft.vc13_86, self.soft.vc22_64, self.soft.vc22_86, self.soft.dx].iter().enumerate() {
            if *installed { self.install_state[i].store(1, Ordering::SeqCst); }
        }
    }

    fn toast(&mut self, s: &str) { self.msg = s.into(); self.msg_t = 2.5; self.anim.toast = 0.0; }

    fn run_cmd_bg(&mut self, cmd: &str, args: &[&str], toast_msg: &str, reload: bool) {
        if self.cmd_busy.load(Ordering::SeqCst) { return; }
        self.cmd_busy.store(true, Ordering::SeqCst);
        self.toast("Running...");
        let busy = self.cmd_busy.clone();
        let needs_reload = self.needs_reload.clone();
        let pending_toast = self.pending_toast.clone();
        let cmd = cmd.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let msg = toast_msg.to_string();
        std::thread::spawn(move || {
            let _ = Command::new(&cmd).args(args.iter().map(|s| s.as_str()).collect::<Vec<_>>()).creation_flags(CREATE_NO_WINDOW).output();
            if let Ok(mut t) = pending_toast.lock() { *t = msg; }
            if reload { needs_reload.store(true, Ordering::SeqCst); }
            busy.store(false, Ordering::SeqCst);
        });
    }

    fn start_install(&self, idx: usize, url: &str, args: &str) {
        let state = self.install_state[idx].clone();
        state.store(2, Ordering::SeqCst);
        let url = url.to_string();
        let args = args.to_string();
        std::thread::spawn(move || {
            let ps = format!(
                "$f = Join-Path $env:TEMP 'installer_{}.exe'; Invoke-WebRequest -Uri '{}' -OutFile $f; Start-Process -FilePath $f -ArgumentList '{}' -Wait; Remove-Item $f -Force",
                idx, url, args
            );
            let _ = Command::new("powershell").args(["-Command", &ps]).creation_flags(CREATE_NO_WINDOW).output();
            state.store(3, Ordering::SeqCst);
        });
    }

    fn export_info(&self) -> String {
        let mut out = String::new();
        out.push_str("=== Emporium Support Export ===\n\n");
        out.push_str("[System]\n");
        out.push_str(&format!("OS: {}\n", self.sys.os));
        out.push_str(&format!("CPU: {}\n", self.sys.cpu));
        out.push_str(&format!("RAM: {}\n", self.sys.ram));
        if !self.sys.mb.trim().is_empty() { out.push_str(&format!("Motherboard: {}\n", self.sys.mb)); }
        for (i, g) in self.sys.gpu.iter().enumerate() { out.push_str(&format!("GPU {}: {}\n", i+1, g)); }
        out.push_str("\n[Storage]\n");
        for (m, f, t) in &self.sys.drives { out.push_str(&format!("{} {} free / {}\n", m, f, t)); }
        if !self.sys.macs.is_empty() {
            out.push_str("\n[Network]\n");
            for (n, m) in &self.sys.macs { out.push_str(&format!("{}: {}\n", n, m)); }
        }
        out.push_str("\n[HWID]\n");
        out.push_str(&format!("CPU ID: {}\n", self.hwid.cpu));
        out.push_str(&format!("MB Serial: {}\n", self.hwid.mb));
        out.push_str(&format!("BIOS: {}\n", self.hwid.bios));
        out.push_str(&format!("GUID: {}\n", self.hwid.guid));
        out.push_str(&format!("Product ID: {}\n", self.hwid.pid));
        out.push_str(&format!("UUID: {}\n", self.hwid.uuid));
        for (m, s) in &self.hwid.disks { out.push_str(&format!("Disk {}: {}\n", m, s)); }
        out.push_str("\n[Security]\n");
        out.push_str(&format!("UAC: {} | Defender: {} | Real-Time: {} | SmartScreen: {} | Secure Boot: {}\n",
            if self.win.uac { "ON" } else { "OFF" },
            if self.win.def { "ON" } else { "OFF" },
            if self.win.rt { "ON" } else { "OFF" },
            if self.win.ss { "ON" } else { "OFF" },
            if self.win.sb { "ON" } else { "OFF" },
        ));
        out.push_str("\n[Installs]\n");
        for (name, installed) in [
            ("VC++ 2013 x64", self.soft.vc13_64), ("VC++ 2013 x86", self.soft.vc13_86),
            ("VC++ 2022 x64", self.soft.vc22_64), ("VC++ 2022 x86", self.soft.vc22_86),
            ("DirectX", self.soft.dx),
        ] {
            out.push_str(&format!("{}: {}\n", name, if installed { "OK" } else { "MISSING" }));
        }
        out.push_str(&format!("\nAdmin: {}\n", if self.is_admin { "Yes" } else { "No" }));
        out
    }
}

fn progs() -> Vec<String> {
    let mut p = vec![];
    for path in [r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"] {
        if let Ok(k) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
            for n in k.enum_keys().filter_map(|x| x.ok()) { if let Ok(sk) = k.open_subkey(&n) { if let Ok(v) = sk.get_value::<String,_>("DisplayName") { p.push(v); } } }
        }
    }
    p
}

fn row(ui: &mut egui::Ui, l: &str, v: &str, cp: bool) -> bool {
    let mut c = false;
    let display_val = if v.trim().is_empty() { "N/A" } else { v };
    let is_na = v.trim().is_empty();
    ui.horizontal(|ui| {
        ui.add_sized([95.0, 20.0], egui::Label::new(RichText::new(l).color(DIM).size(11.0)));
        ui.add(egui::Label::new(RichText::new(display_val).color(if is_na { DIM } else { TXT }).size(11.0)).truncate());
        if cp && !is_na {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(RichText::new("⎘").size(11.0).color(DIM)).fill(Color32::TRANSPARENT).min_size(Vec2::new(20.0, 20.0))).on_hover_text("Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = v.into()); c = true;
                }
            });
        }
    });
    c
}

fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        let t = ui.input(|i| i.time) as f32;
        let dot_alpha = 0.6 + (t * 3.0).sin() * 0.4;
        let dot_rect = ui.available_rect_before_wrap();
        ui.painter().circle_filled(egui::pos2(dot_rect.left() + 4.0, dot_rect.center().y), 3.0, ACCENT.gamma_multiply(dot_alpha));
        ui.add_space(12.0);
        ui.label(RichText::new(label).color(ACCENT).size(10.0).strong());
        ui.add_space(8.0);
        let r = ui.available_rect_before_wrap();
        let line_y = r.center().y;
        let line_end = r.right() - 10.0;
        let line_start = r.left();
        for i in 0..((line_end - line_start) as i32 / 3) {
            let x = line_start + (i * 3) as f32;
            let alpha = 1.0 - (i as f32 / ((line_end - line_start) / 3.0)).min(1.0);
            ui.painter().line_segment(
                [egui::pos2(x, line_y), egui::pos2(x + 2.0, line_y)],
                Stroke::new(1.0, BORDER.gamma_multiply(alpha))
            );
        }
    });
    ui.add_space(6.0);
}

fn badge(ui: &mut egui::Ui, ok: bool) {
    let t = ui.input(|i| i.time) as f32;
    let pulse = if !ok { 0.8 + (t * 4.0).sin().abs() * 0.2 } else { 1.0 };
    let (txt, bg, fg) = if ok { ("OK", GRN.gamma_multiply(0.2), GRN) } else { ("NO", RED.gamma_multiply(0.2 * pulse), RED.gamma_multiply(pulse)) };
    egui::Frame::none().fill(bg).rounding(4.0).inner_margin(egui::vec2(8.0, 3.0)).show(ui, |ui| {
        ui.label(RichText::new(txt).color(fg).size(9.0).strong());
    });
}

fn badge_inv(ui: &mut egui::Ui, on: bool) {
    let t = ui.input(|i| i.time) as f32;
    let pulse = if on { 0.8 + (t * 4.0).sin().abs() * 0.2 } else { 1.0 };
    let (txt, bg, fg) = if on { ("ON", RED.gamma_multiply(0.2 * pulse), RED.gamma_multiply(pulse)) } else { ("OFF", GRN.gamma_multiply(0.2), GRN) };
    egui::Frame::none().fill(bg).rounding(4.0).inner_margin(egui::vec2(8.0, 3.0)).show(ui, |ui| {
        ui.label(RichText::new(txt).color(fg).size(9.0).strong());
    });
}

fn progress_bar(ui: &mut egui::Ui, progress: f32, color: Color32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 4.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 2.0, CARD2);
    let filled_width = rect.width() * progress;
    let filled = egui::Rect::from_min_size(rect.min, Vec2::new(filled_width, rect.height()));
    ui.painter().rect_filled(filled, 2.0, color);
    let t = ui.input(|i| i.time) as f32;
    let shimmer_pos = ((t * 0.8) % 1.5 - 0.25) * rect.width();
    if shimmer_pos > 0.0 && shimmer_pos < filled_width {
        let shimmer_rect = egui::Rect::from_min_size(
            egui::pos2(rect.left() + shimmer_pos - 10.0, rect.top()),
            Vec2::new(20.0, rect.height())
        ).intersect(filled);
        if shimmer_rect.width() > 0.0 {
            ui.painter().rect_filled(shimmer_rect, 2.0, Color32::WHITE.gamma_multiply(0.3));
        }
    }
}

fn spawn_cmd(cmd: &str, args: &[&str]) { let _ = Command::new(cmd).args(args).creation_flags(CREATE_NO_WINDOW).spawn(); }

fn btn(ui: &mut egui::Ui, t: &str, accent: bool) -> bool {
    let base_fill = if accent { ACCENT } else { CARD2 };
    let txt_color = if accent { Color32::WHITE } else { TXT };
    let response = ui.add(egui::Button::new(RichText::new(t).size(10.0).color(txt_color)).fill(base_fill).stroke(Stroke::new(1.0, if accent { ACCENT } else { BORDER })).rounding(4.0).min_size(Vec2::new(0.0, 24.0)));
    if response.hovered() {
        let glow_color = if accent { ACCENT } else { TXT };
        ui.painter().rect_filled(response.rect.expand(2.0), 6.0, glow_color.gamma_multiply(0.1));
    }
    response.clicked()
}

fn btn_wide(ui: &mut egui::Ui, t: &str, accent: bool) -> bool {
    let base_fill = if accent { ACCENT } else { CARD2 };
    let txt_color = if accent { Color32::WHITE } else { TXT };
    let response = ui.add_sized([ui.available_width(), 26.0], egui::Button::new(RichText::new(t).size(10.0).color(txt_color)).fill(base_fill).stroke(Stroke::new(1.0, if accent { ACCENT } else { BORDER })).rounding(4.0));
    if response.hovered() {
        let glow_color = if accent { ACCENT } else { TXT };
        ui.painter().rect_filled(response.rect.expand(2.0), 6.0, glow_color.gamma_multiply(0.1));
    }
    response.clicked()
}

impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] { [0.0;4] }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.load_icons(ctx);
        let dt = ctx.input(|i| i.predicted_dt);

        // Handle background command completions
        if let Ok(mut t) = self.pending_toast.try_lock() {
            if !t.is_empty() {
                self.msg = t.clone();
                self.msg_t = 2.5;
                self.anim.toast = 0.0;
                t.clear();
            }
        }
        if self.needs_reload.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
            self.load();
        }

        // Update animations
        self.anim.global_time += dt;

        if self.anim.tab < 1.0 {
            self.anim.tab = (self.anim.tab + dt * 6.0).min(1.0);
        }

        if self.msg_t > 0.0 {
            self.anim.toast = (self.anim.toast + dt * 8.0).min(1.0);
            self.msg_t -= dt;
            if self.msg_t <= 0.0 { self.msg.clear(); self.anim.toast = 0.0; }
        }

        if self.anim.refresh_spin > 0.0 {
            self.anim.refresh_spin = (self.anim.refresh_spin - dt * 3.0).max(0.0);
        }

        let any_installing = self.install_state.iter().any(|s| s.load(Ordering::SeqCst) == 2);

        if self.auto_refresh {
            self.refresh_timer -= dt;
            if self.refresh_timer <= 0.0 {
                self.load();
                self.refresh_timer = 30.0;
            }
        }

        // Request repaint for animations
        ctx.request_repaint();

        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let r = ui.max_rect();
            ui.painter().rect_filled(r, 8.0, BG);
            let border_pulse = 0.6 + (self.anim.global_time * 1.5).sin().abs() * 0.4;
            let border_color = Color32::from_rgba_unmultiplied(
                (BORDER.r() as f32 + (ACCENT.r() as f32 - BORDER.r() as f32) * 0.1 * border_pulse) as u8,
                (BORDER.g() as f32 + (ACCENT.g() as f32 - BORDER.g() as f32) * 0.1 * border_pulse) as u8,
                (BORDER.b() as f32 + (ACCENT.b() as f32 - BORDER.b() as f32) * 0.1 * border_pulse) as u8,
                255
            );
            ui.painter().rect_stroke(r, 8.0, Stroke::new(1.0, border_color));

            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(r.shrink(1.0)), |ui| {
                ui.vertical(|ui| {
                    // Title bar
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.label(RichText::new("Emporium").size(17.0).color(TXT).strong());
                        ui.label(RichText::new("Support").size(17.0).color(DIM));
                        ui.add_space(6.0);
                        // Admin badge
                        if self.is_admin {
                            egui::Frame::none().fill(GRN.gamma_multiply(0.15)).rounding(4.0).inner_margin(egui::vec2(6.0, 2.0)).show(ui, |ui| {
                                ui.label(RichText::new("ADMIN").color(GRN).size(8.0).strong());
                            });
                        } else {
                            egui::Frame::none().fill(YLW.gamma_multiply(0.15)).rounding(4.0).inner_margin(egui::vec2(6.0, 2.0)).show(ui, |ui| {
                                ui.label(RichText::new("USER").color(YLW).size(8.0).strong());
                            });
                        }
                        ui.add_space(8.0);
                        // Refresh button
                        let refresh_char = if self.anim.refresh_spin > 0.0 {
                            let idx = ((self.anim.global_time * 12.0) as usize) % 4;
                            ["◴", "◷", "◶", "◵"][idx]
                        } else { "⟳" };
                        if ui.add(egui::Button::new(RichText::new(refresh_char).size(14.0).color(if self.anim.refresh_spin > 0.0 { ACCENT } else { DIM })).fill(Color32::TRANSPARENT).frame(false)).on_hover_text("Refresh").clicked() {
                            self.load();
                            self.refresh_timer = 30.0;
                            self.anim.refresh_spin = 1.0;
                            self.toast("Refreshed");
                        }
                        // Auto-refresh toggle
                        let auto_color = if self.auto_refresh { GRN } else { DIM };
                        let auto_tip = if self.auto_refresh { format!("Auto-refresh ON ({:.0}s)", self.refresh_timer) } else { "Auto-refresh OFF".to_string() };
                        if ui.add(egui::Button::new(RichText::new("⏱").size(13.0).color(auto_color)).fill(Color32::TRANSPARENT).frame(false)).on_hover_text(auto_tip).clicked() {
                            self.auto_refresh = !self.auto_refresh;
                            self.refresh_timer = 30.0;
                            self.toast(if self.auto_refresh { "Auto-refresh ON (30s)" } else { "Auto-refresh OFF" });
                        }
                        // Export button
                        if ui.add(egui::Button::new(RichText::new("📋").size(13.0).color(DIM)).fill(Color32::TRANSPARENT).frame(false)).on_hover_text("Export all info to clipboard").clicked() {
                            let info = self.export_info();
                            ui.output_mut(|o| o.copied_text = info);
                            self.toast("Exported to clipboard");
                        }
                        // Busy spinner
                        if self.cmd_busy.load(Ordering::SeqCst) {
                            let idx = ((self.anim.global_time * 8.0) as usize) % 4;
                            ui.label(RichText::new(["◐", "◓", "◑", "◒"][idx]).color(YLW).size(13.0));
                        }
                        let drag = ui.interact(ui.available_rect_before_wrap(), ui.id().with("d"), egui::Sense::drag());
                        if drag.dragged() { ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag); }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_space(10.0);
                            let close_btn = ui.add(egui::Button::new(RichText::new("✕").size(13.0).color(if ui.rect_contains_pointer(ui.min_rect()) { RED } else { DIM })).fill(Color32::TRANSPARENT).min_size(Vec2::new(30.0, 24.0)));
                            if close_btn.hovered() {
                                let t = self.anim.global_time;
                                let pulse = 0.3 + (t * 6.0).sin().abs() * 0.1;
                                ui.painter().rect_filled(close_btn.rect.expand(2.0), 6.0, RED.gamma_multiply(pulse));
                            }
                            if close_btn.clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                            let min_btn = ui.add(egui::Button::new(RichText::new("—").size(13.0).color(DIM)).fill(Color32::TRANSPARENT).min_size(Vec2::new(30.0, 24.0)));
                            if min_btn.hovered() {
                                ui.painter().rect_filled(min_btn.rect.expand(1.0), 4.0, DIM.gamma_multiply(0.2));
                            }
                            if min_btn.clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)); }
                        });
                    });
                    ui.add_space(8.0);

                    // Tabs with icons
                    let tab_height = 26.0;
                    ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), tab_height), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(14.0);
                        ui.spacing_mut().item_spacing.x = 2.0;
                        for t in Tab::all() {
                            let sel = *t == self.tab;
                            let ico_texture = self.icons.as_ref().map(|icons| match t {
                                Tab::System => &icons.pc,
                                Tab::Hwid => &icons.key,
                                Tab::Windows => &icons.gear,
                                Tab::Boosters => &icons.rocket,
                                Tab::AntiCheat => &icons.shield,
                                Tab::Installs => &icons.down,
                                Tab::Controls => &icons.menu,
                            });
                            let start_x = ui.cursor().left();
                            if let Some(ico) = ico_texture {
                                ui.add_sized([11.0, tab_height], egui::Image::new(ico).fit_to_exact_size(Vec2::new(11.0, 11.0)).tint(if sel { ACCENT } else { DIM }));
                            }
                            ui.add_space(2.0);
                            let btn_response = ui.add(egui::Button::new(RichText::new(t.name()).size(11.0).color(if sel { TXT } else { DIM })).fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(0.0).min_size(Vec2::new(0.0, tab_height)));
                            let end_x = ui.cursor().left();
                            // Tab hover highlight
                            if btn_response.hovered() && !sel {
                                let hover_rect = egui::Rect::from_min_max(egui::pos2(start_x, ui.min_rect().top()), egui::pos2(end_x, ui.min_rect().bottom()));
                                ui.painter().rect_filled(hover_rect, 4.0, TXT.gamma_multiply(0.04));
                            }
                            if sel {
                                let eased = ease_out_back(self.anim.tab);
                                let total_w = end_x - start_x - 4.0;
                                let w = total_w * eased;
                                let h = 3.0 * eased;
                                let center_x = start_x + (end_x - start_x) / 2.0;
                                let bottom_y = ui.min_rect().bottom();
                                ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(center_x - w/2.0, bottom_y - h), egui::vec2(w, h)), 1.5, ACCENT);
                            }
                            if btn_response.clicked() && !sel {
                                let old_idx = Tab::all().iter().position(|x| *x == self.tab).unwrap_or(0);
                                let new_idx = Tab::all().iter().position(|x| x == t).unwrap_or(0);
                                self.anim.tab_dir = if new_idx > old_idx { 1 } else { -1 };
                                self.tab = *t;
                                self.anim.tab = 0.0;
                            }
                            ui.add_space(6.0);
                        }
                    });

                    // Separator
                    let sep = ui.available_rect_before_wrap();
                    ui.painter().line_segment([egui::pos2(sep.left() + 12.0, sep.top()), egui::pos2(sep.right() - 12.0, sep.top())], Stroke::new(1.0, BORDER));
                    ui.add_space(10.0);

                    // Content with slide animation
                    egui::ScrollArea::vertical().auto_shrink([false,true]).show(ui, |ui| {
                        let eased = ease_out_cubic(self.anim.tab);
                        let slide_offset = (1.0 - eased) * 30.0 * self.anim.tab_dir as f32;
                        ui.set_opacity(eased);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0 + slide_offset);
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width() - 28.0);
                                let sys = self.sys.clone(); let hwid = self.hwid.clone(); let win = self.win; let soft = self.soft.clone();

                                match self.tab {
                                    Tab::System => {
                                        section_header(ui, "HARDWARE");
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            if row(ui, "OS", &sys.os, true) { self.toast("Copied"); }
                                            if row(ui, "CPU", &sys.cpu, true) { self.toast("Copied"); }
                                            if row(ui, "RAM", &sys.ram, false) {}
                                            if !sys.mb.trim().is_empty() { if row(ui, "Motherboard", &sys.mb, true) { self.toast("Copied"); } }
                                            for (i,g) in sys.gpu.iter().enumerate() { if row(ui, &format!("GPU {}", i+1), g, true) { self.toast("Copied"); } }
                                        });

                                        section_header(ui, "STORAGE");
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            for (mount, free, total) in &sys.drives {
                                                let free_num: f64 = free.trim_end_matches('G').parse().unwrap_or(0.0);
                                                let total_num: f64 = total.trim_end_matches('G').parse().unwrap_or(1.0);
                                                let usage = 1.0 - (free_num / total_num);
                                                ui.horizontal(|ui| {
                                                    ui.label(RichText::new(mount).color(CYAN).size(12.0).strong());
                                                    ui.add_space(8.0);
                                                    ui.label(RichText::new(format!("{} free", free)).color(GRN).size(10.0));
                                                    ui.label(RichText::new(format!("/ {}", total)).color(DIM).size(10.0));
                                                });
                                                progress_bar(ui, usage as f32, if usage > 0.9 { RED } else if usage > 0.7 { YLW } else { ACCENT });
                                                ui.add_space(6.0);
                                            }
                                        });

                                        if !sys.macs.is_empty() {
                                            section_header(ui, "NETWORK");
                                            egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                for (n,m) in &sys.macs { if row(ui, n, m, true) { self.toast("Copied"); } }
                                            });
                                        }
                                    }
                                    Tab::Hwid => {
                                        section_header(ui, "IDENTIFIERS");
                                        // Copy All button
                                        if btn_wide(ui, "📋 Copy All HWID Info", true) {
                                            let mut all = String::new();
                                            all.push_str(&format!("CPU ID: {}\n", hwid.cpu));
                                            all.push_str(&format!("MB Serial: {}\n", hwid.mb));
                                            all.push_str(&format!("BIOS: {}\n", hwid.bios));
                                            all.push_str(&format!("GUID: {}\n", hwid.guid));
                                            all.push_str(&format!("Product ID: {}\n", hwid.pid));
                                            all.push_str(&format!("UUID: {}\n", hwid.uuid));
                                            for (m, s) in &hwid.disks { all.push_str(&format!("Disk {}: {}\n", m, s)); }
                                            ui.output_mut(|o| o.copied_text = all);
                                            self.toast("All HWID copied");
                                        }
                                        ui.add_space(6.0);
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            if row(ui, "CPU ID", &hwid.cpu, true) { self.toast("Copied"); }
                                            if row(ui, "MB Serial", &hwid.mb, true) { self.toast("Copied"); }
                                            if row(ui, "BIOS", &hwid.bios, true) { self.toast("Copied"); }
                                            if row(ui, "GUID", &hwid.guid, true) { self.toast("Copied"); }
                                            if row(ui, "Product ID", &hwid.pid, true) { self.toast("Copied"); }
                                            if row(ui, "UUID", &hwid.uuid, true) { self.toast("Copied"); }
                                        });

                                        section_header(ui, "DISK SERIALS");
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            if hwid.disks.is_empty() {
                                                ui.label(RichText::new("No disk serials found (may require admin)").color(DIM).size(10.0));
                                            } else {
                                                for (m,s) in &hwid.disks { if row(ui, m, s, true) { self.toast("Copied"); } }
                                            }
                                        });
                                    }
                                    Tab::Windows => {
                                        section_header(ui, "SECURITY SETTINGS");
                                        if !self.is_admin {
                                            egui::Frame::none().fill(YLW.gamma_multiply(0.1)).rounding(4.0).inner_margin(egui::vec2(10.0, 6.0)).show(ui, |ui| {
                                                ui.label(RichText::new("⚠ Run as Administrator for changes to apply").color(YLW).size(10.0));
                                            });
                                            ui.add_space(6.0);
                                        }
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            let cmd_busy = self.cmd_busy.load(Ordering::SeqCst);
                                            for (name, val, on_cmd, off_cmd) in [
                                                ("UAC", win.uac, "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System' -Name 'EnableLUA' -Value 1", "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System' -Name 'EnableLUA' -Value 0"),
                                                ("Real-Time Protection", win.rt, "Remove-ItemProperty -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows Defender\\Real-Time Protection' -Name 'DisableRealtimeMonitoring' -ErrorAction SilentlyContinue; Set-MpPreference -DisableRealtimeMonitoring $false", "New-Item -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows Defender\\Real-Time Protection' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows Defender\\Real-Time Protection' -Name 'DisableRealtimeMonitoring' -Value 1 -Type DWord; Set-MpPreference -DisableRealtimeMonitoring $true"),
                                                ("SmartScreen", win.ss, "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer' -Name 'SmartScreenEnabled' -Value 'RequireAdmin'; Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows\\System' -Name 'EnableSmartScreen' -Value 1 -Type DWord -ErrorAction SilentlyContinue", "Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer' -Name 'SmartScreenEnabled' -Value 'Off'; New-Item -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows\\System' -Force | Out-Null; Set-ItemProperty -Path 'HKLM:\\SOFTWARE\\Policies\\Microsoft\\Windows\\System' -Name 'EnableSmartScreen' -Value 0 -Type DWord"),
                                            ] {
                                                ui.horizontal(|ui| {
                                                    ui.add_sized([120.0, 22.0], egui::Label::new(RichText::new(name).color(TXT).size(11.0)));
                                                    badge_inv(ui, val);
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if cmd_busy { ui.disable(); }
                                                        if btn(ui, "Off", false) { self.run_cmd_bg("powershell", &["-Command", off_cmd], "Done - May need restart", true); }
                                                        if btn(ui, "On", false) { self.run_cmd_bg("powershell", &["-Command", on_cmd], "Done", true); }
                                                    });
                                                });
                                                ui.add_space(4.0);
                                            }
                                            ui.horizontal(|ui| { ui.add_sized([120.0, 22.0], egui::Label::new(RichText::new("Defender").color(TXT).size(11.0))); badge_inv(ui, win.def); });
                                            ui.add_space(4.0);
                                            ui.horizontal(|ui| { ui.add_sized([120.0, 22.0], egui::Label::new(RichText::new("Secure Boot").color(TXT).size(11.0))); badge_inv(ui, win.sb); });
                                        });
                                    }
                                    Tab::Boosters => {
                                        section_header(ui, "GAME BOOSTERS");
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            for (n,v) in [("Razer Cortex", soft.razer), ("Dragon Center", soft.dragon), ("MSI Afterburner", soft.ab)] {
                                                ui.horizontal(|ui| { ui.add_sized([160.0, 22.0], egui::Label::new(RichText::new(n).color(TXT).size(11.0))); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { badge_inv(ui, v); }); });
                                                ui.add_space(4.0);
                                            }
                                        });
                                    }
                                    Tab::AntiCheat => {
                                        section_header(ui, "ANTI-CHEAT SOFTWARE");
                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            for (n,v) in [("Riot Vanguard", soft.vg), ("FaceIT AC", soft.faceit), ("ESEA Client", soft.esea), ("EasyAntiCheat", soft.eac), ("BattlEye", soft.be), ("Malwarebytes", soft.mb)] {
                                                ui.horizontal(|ui| { ui.add_sized([160.0, 22.0], egui::Label::new(RichText::new(n).color(TXT).size(11.0))); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { badge_inv(ui, v); }); });
                                                ui.add_space(4.0);
                                            }
                                        });
                                    }
                                    Tab::Installs => {
                                        let items = [
                                            ("VC++ 2013 x64", soft.vc13_64, "https://aka.ms/highdpimfc2013x64enu", "/install /quiet /norestart"),
                                            ("VC++ 2013 x86", soft.vc13_86, "https://aka.ms/highdpimfc2013x86enu", "/install /quiet /norestart"),
                                            ("VC++ 2022 x64", soft.vc22_64, "https://aka.ms/vs/17/release/vc_redist.x64.exe", "/install /quiet /norestart"),
                                            ("VC++ 2022 x86", soft.vc22_86, "https://aka.ms/vs/17/release/vc_redist.x86.exe", "/install /quiet /norestart"),
                                            ("DirectX Runtime", soft.dx, "https://download.microsoft.com/download/1/7/1/1718CCC4-6315-4D8E-9543-8E28A4E18C4C/dxwebsetup.exe", "/Q"),
                                        ];

                                        section_header(ui, "REQUIRED INSTALLS");

                                        let missing: Vec<_> = items.iter().enumerate().filter(|(_, (_, v, _, _))| !v).collect();
                                        if !missing.is_empty() {
                                            if btn_wide(ui, &format!("⬇ Install All Missing ({})", missing.len()), true) {
                                                for (idx, (_, _, url, args)) in &missing {
                                                    self.start_install(*idx, url, args);
                                                }
                                                self.toast("Installing all...");
                                            }
                                            ui.add_space(8.0);
                                        }

                                        egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(12.0).show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            for (idx, (name, installed, url, args)) in items.iter().enumerate() {
                                                let state = self.install_state[idx].load(Ordering::SeqCst);
                                                ui.horizontal(|ui| {
                                                    ui.add_sized([130.0, 22.0], egui::Label::new(RichText::new(*name).color(TXT).size(11.0)));

                                                    match state {
                                                        1 => { badge(ui, true); }
                                                        2 => {
                                                            let t = ui.input(|i| i.time) as f32;
                                                            let spinner = ["◐", "◓", "◑", "◒"][(t * 8.0) as usize % 4];
                                                            ui.label(RichText::new(spinner).color(YLW).size(12.0));
                                                            ui.label(RichText::new("Installing...").color(YLW).size(10.0));
                                                        }
                                                        3 => {
                                                            ui.label(RichText::new("✓ Done").color(GRN).size(10.0));
                                                        }
                                                        _ => { badge(ui, false); }
                                                    }

                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        if !installed && state != 2 {
                                                            if btn(ui, "Install", true) {
                                                                self.start_install(idx, url, args);
                                                                self.toast(&format!("Installing {}...", name));
                                                            }
                                                        }
                                                    });
                                                });

                                                if state == 2 {
                                                    ui.add_space(2.0);
                                                    let t = ui.input(|i| i.time) as f32;
                                                    let fake_progress = ((t * 0.3).sin() * 0.3 + 0.5).min(0.95);
                                                    progress_bar(ui, fake_progress as f32, ACCENT);
                                                }
                                                ui.add_space(6.0);
                                            }
                                        });
                                    }
                                    Tab::Controls => {
                                        let cmd_busy = self.cmd_busy.load(Ordering::SeqCst);
                                        ui.horizontal(|ui| {
                                            ui.vertical(|ui| {
                                                ui.set_width((ui.available_width() - 16.0) / 2.0);
                                                section_header(ui, "COMMANDS");
                                                egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(10.0).show(ui, |ui| {
                                                    ui.set_width(ui.available_width());
                                                    if btn_wide(ui, "Install Hyper-V", true) { spawn_cmd("powershell", &["-Command", "Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All"]); self.toast("Installing..."); }
                                                    ui.add_space(3.0);
                                                    ui.add_enabled_ui(!cmd_busy, |ui| {
                                                        if btn_wide(ui, "Enable Hyper-V", false) { self.run_cmd_bg("bcdedit", &["/set", "hypervisorlaunchtype", "auto"], "Done - Restart needed", false); }
                                                        ui.add_space(3.0);
                                                        if btn_wide(ui, "Disable Hyper-V", false) { self.run_cmd_bg("bcdedit", &["/set", "hypervisorlaunchtype", "off"], "Done - Restart needed", false); }
                                                        ui.add_space(6.0);
                                                        if btn_wide(ui, "Driver (NAL) Fix", true) { self.run_cmd_bg("powershell", &["-Command", "sc.exe delete iqvw64e.sys; Remove-Item 'C:\\Windows\\System32\\drivers\\iqvw64e.sys' -Force -ErrorAction SilentlyContinue"], "Done", false); }
                                                        ui.add_space(3.0);
                                                        if btn_wide(ui, "Block Win Updates", false) { self.run_cmd_bg("powershell", &["-Command", "Stop-Service wuauserv; Set-Service wuauserv -StartupType Disabled"], "Updates Blocked", false); }
                                                        ui.add_space(3.0);
                                                        if btn_wide(ui, "Network Reset", false) { self.run_cmd_bg("powershell", &["-Command", "ipconfig /flushdns; netsh winsock reset; netsh int ip reset"], "Done - Restart needed", false); }
                                                    });
                                                    ui.add_space(6.0);
                                                    ui.horizontal(|ui| {
                                                        if btn(ui, "DevMgr", false) { spawn_cmd("mmc", &["devmgmt.msc"]); }
                                                        if btn(ui, "Services", false) { spawn_cmd("mmc", &["services.msc"]); }
                                                        if btn(ui, "Disks", false) { spawn_cmd("mmc", &["diskmgmt.msc"]); }
                                                    });
                                                });
                                            });
                                            ui.add_space(16.0);
                                            ui.vertical(|ui| {
                                                section_header(ui, "DOWNLOADS");
                                                egui::Frame::none().fill(CARD).rounding(6.0).stroke(Stroke::new(1.0, BORDER)).inner_margin(10.0).show(ui, |ui| {
                                                    ui.set_width(ui.available_width());
                                                    for (n,u) in [("WinRAR","https://www.rarlab.com/download.htm"),("7-Zip","https://www.7-zip.org/download.html"),("Revo Uninstaller","https://www.revouninstaller.com/revo-uninstaller-free-download/"),("Win10 ISO","https://www.microsoft.com/en-us/software-download/windows10ISO"),("Win11 ISO","https://www.microsoft.com/en-us/software-download/windows11"),("All VC++ Redist","https://www.techpowerup.com/download/visual-c-redistributable-runtime-package-all-in-one/"),("NVIDIA Drivers","https://www.nvidia.com/Download/index.aspx"),("AMD Drivers","https://www.amd.com/en/support")] {
                                                        if btn_wide(ui, n, true) { let _ = open::that(u); }
                                                        ui.add_space(3.0);
                                                    }
                                                });
                                            });
                                        });
                                    }
                                }
                            });
                            ui.add_space(14.0);
                        });
                    });

                    // Toast notification with fade-out
                    if !self.msg.is_empty() {
                        let toast_eased = ease_out_back(self.anim.toast);
                        let fade = if self.msg_t < 0.5 { (self.msg_t / 0.5).max(0.0) } else { 1.0 };
                        let alpha = toast_eased * fade;
                        let slide_up = (1.0 - toast_eased) * 20.0;
                        ui.add_space(4.0 + slide_up);
                        ui.horizontal(|ui| {
                            ui.add_space(14.0);
                            ui.set_opacity(alpha);
                            let glow_intensity = 0.15 + (self.anim.global_time * 4.0).sin().abs() * 0.05;
                            egui::Frame::none().fill(ACCENT.gamma_multiply(glow_intensity)).rounding(6.0).inner_margin(egui::vec2(12.0, 6.0)).show(ui, |ui| {
                                // Show spinner if command is running
                                if self.cmd_busy.load(Ordering::SeqCst) && self.msg == "Running..." {
                                    let idx = ((self.anim.global_time * 8.0) as usize) % 4;
                                    ui.label(RichText::new(["◐", "◓", "◑", "◒"][idx]).color(YLW).size(11.0));
                                }
                                ui.label(RichText::new(&self.msg).color(ACCENT).size(11.0).strong());
                            });
                        });
                    }
                    ui.add_space(8.0);
                });
            });
        });

        // Suppress unused variable warnings
        let _ = any_installing;
    }
}
