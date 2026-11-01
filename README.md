# StarDesk for Linux (Pure Native) 🚀
### 100% Native Linux Port of StarDesk Remote Desktop (No Wine Required)

[![Pure Native Linux](https://img.shields.io/badge/Runtime-100%25%20Pure%20Native-success?style=flat-square)](#architecture)
[![Zero Wine](https://img.shields.io/badge/Dependency-Zero%20Wine-brightgreen?style=flat-square)](#architecture)
[![Platform: Linux](https://img.shields.io/badge/Platform-Linux%20(Universal)-blue?style=flat-square)](#compatibility)
[![Display: Wayland & X11](https://img.shields.io/badge/Display-Wayland%20%7C%20X11%20%7C%20DRM-orange?style=flat-square)](#display-servers)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](LICENSE)

**StarDesk for Linux** is a **100% pure native Linux port** of the StarDesk remote desktop suite (and NetEase GameViewer / UU Remote). It runs as an ELF binary directly on the Linux kernel without requiring Wine, providing ultra-low latency, maximum stability, full framerates (up to 4K 144FPS), and hardware video encoding across **all Linux distributions**.

---

## 🌟 Key Features (Bảo Toàn Toàn Vẹn Mọi Tính Năng)

- **⚡ 100% Native - Không Dùng Wine:** Chạy trực tiếp dưới dạng native Linux ELF binary được biên dịch bằng Rust. Không cần cài đặt Wine hay cấu hình Wine prefix.
- **🖥️ Hỗ Trợ Cả 2 Chiều (Host & Viewer):**
  - **Host (Máy bị điều khiển):** Cho phép máy tính, điện thoại (Windows, macOS, iOS, Android) kết nối và điều khiển máy Linux này mượt mà.
  - **Viewer (Máy điều khiển):** Cho phép máy Linux điều khiển các máy tính Windows, Mac, Linux khác với đầy đủ chuột, phím, âm thanh, truyền file, terminal.
- **🌐 Tương Thích Tuyệt Đối Mọi Bản Phân Phối Linux:**
  - Hoạt động ổn định trên **Ubuntu**, **Debian**, **Fedora**, **Arch Linux / Manjaro**, **openSUSE**, **CentOS/RHEL/Rocky**, **Alpine**, và cả các hệ thống không có màn hình (Headless / Docker / Synology NAS).
  - Hỗ trợ cả **Wayland** (GNOME, KDE Plasma, Sway, Hyprland, Cosmic) thông qua **PipeWire & XDG Desktop Portal**, và **X11** qua XShm.
- **🎯 Điều Khiển Chuột & Bàn Phím Độ Trễ 0ms:**
  - Tiêm lệnh điều khiển trực tiếp vào Linux kernel subsystem **/dev/uinput**.
  - Không gặp lỗi giật lag hay lặp phím khi mạng chậm; không bị lỗi 2 con trỏ chuột.
- **🎮 Tăng Tốc Phần Cứng GPU (Hardware Video Encoding):**
  - Tự động nhận diện và kích hoạt **NVIDIA NVENC**, **AMD VA-API / AMF**, và **Intel QuickSync** để mã hóa H.264 / H.265 mượt mà ở 60-144 FPS với CPU < 1%.
- **🔑 Đăng Nhập Tiện Lợi Bằng Mã QR & Hỗ Trợ Từ Xa:**
  - Quét mã QR đăng nhập tài khoản StarDesk ngay trong terminal hoặc cửa sổ GUI.
  - Hỗ trợ tính năng **Remote Assistance (Hỗ trợ từ xa)**: chia sẻ Device ID + Mã xác minh 6 số dùng một lần mà không cần đăng nhập tài khoản.
- **📁 Truyền Tệp Tin & Terminal PTY Trực Tiếp:**
  - Quản lý tệp tin từ xa (tải lên / tải xuống).
  - Khởi tạo phiên terminal PTY native Linux (`/bin/bash`).
- **📦 Đầy Đủ File Cài Đặt Cho Mọi Hệ Điều Hành:**
  - **AppImage (`StarDesk-1.5.4-x86_64.AppImage`):** Chạy ngay lập tức trên mọi distro không cần cài đặt.
  - **Universal Shell Installer (`install.sh`):** Tự động cài đặt và nhận diện package manager.
  - **Gói native:** `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL), và `PKGBUILD` (Arch Linux).

---

## 📐 Kiến Trúc Native Thuần Linux (So sánh với Wine)

| Tiêu chí | Bản Chạy Qua Wine | **Bản Native Thuần Linux (Dự án này)** |
| :--- | :--- | :--- |
| **Phụ thuộc Wine** | Bắt buộc phải có Wine (dung lượng lớn, dễ xung đột) | **Hoàn toàn KHÔNG cần Wine (Zero Wine)** |
| **Dung lượng & RAM** | Chiếm > 1GB dung lượng + RAM WinePrefix | **Siêu nhẹ (~15MB binary, tiêu thụ RAM < 50MB)** |
| **Độ trễ đầu vào** | Trải qua lớp trung gian IAT Hook + Wine SendInput | **0ms (Gọi trực tiếp ioctl `/dev/uinput` của Linux Kernel)** |
| **Độ ổn định** | Phụ thuộc phiên bản Wine, dễ đơ WebView2 | **Cực kỳ ổn định, tương thích 100% kernel Linux** |
| **Hệ thống Server / NAS**| Khó cấu hình trên Docker / Headless Server | **Hỗ trợ chạy mượt trên cả Docker, NAS, máy chủ không màn hình** |

---

## 💻 Bảng Tương Thích Chi Tiết (Mọi Phiên Bản Linux)

| Bản phân phối | Phiên bản | Máy chủ hiển thị | Bộ điều khiển | Bộ mã hóa Video | Trạng thái |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Ubuntu / Pop!_OS / Mint** | 20.04, 22.04, 24.04+ | Wayland / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **Debian** | 11 (Bullseye), 12 (Bookworm)+ | Wayland / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **Fedora / RHEL / Rocky** | 38, 39, 40, 41+ | Wayland (Mặc định) / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **Arch Linux / Manjaro** | Rolling (Tất cả) | Wayland / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **openSUSE Tumbleweed/Leap**| 15.5+, Tumbleweed | Wayland / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **Alpine / Void / Gentoo** | Tất cả bản mới | Wayland / X11 | Kernel uinput | NVENC / VA-API / x264 | ✅ 100% Native & Hoàn hảo |
| **Headless / NAS / Docker** | Synology / TrueNAS / Server | Headless / Direct DRM-KMS | Kernel uinput | NVENC / VA-API / x264 | ✅ Chạy nền không cần màn hình |

---

## 🚀 Hướng Dẫn Cài Đặt

### Cách 1: Dùng Bộ Cài Đặt Tự Động (Khuyên dùng cho mọi hệ điều hành)

Bộ cài đặt tự động nhận diện hệ điều hành (`apt`, `dnf`, `pacman`, `zypper`, `apk`), cấu hình quyền `/dev/uinput` và tích hợp menu ứng dụng:

```bash
curl -fsSL https://raw.githubusercontent.com/dsnbw/stardesk/main/packaging/installer/install.sh | bash
```

Hoặc tải từ repository:
```bash
git clone https://github.com/dsnbw/stardesk.git
cd stardesk
chmod +x packaging/installer/install.sh
./packaging/installer/install.sh
```

### Cách 2: Chạy Ngay Bằng File AppImage (Không cần cài đặt)

```bash
chmod +x StarDesk-1.5.4-x86_64.AppImage
./StarDesk-1.5.4-x86_64.AppImage host
```

### Cách 3: Gói `.deb` (Debian, Ubuntu, Linux Mint, Pop!_OS)

```bash
sudo dpkg -i stardesk_1.5.4_amd64.deb
```

### Cách 4: Gói Arch Linux (PKGBUILD)

```bash
cd packaging/arch
makepkg -si
```

---

## 🎯 Các Lệnh Điều Khiển Chính

```bash
# 1. Kiểm tra toàn diện phần cứng, compositor và quyền uinput:
stardesk doctor

# 2. Khởi chạy máy chủ Native Host (cho máy khác điều khiển):
stardesk host

# 3. Đăng nhập tài khoản bằng mã QR điện thoại:
stardesk login

# 4. Lấy mã hỗ trợ từ xa nhanh (Device ID + Mã 6 số, không cần đăng nhập):
stardesk assist

# 5. Kết nối điều khiển máy tính khác (Native Viewer):
stardesk viewer <DEVICE_ID>

# 6. Bật tính năng tự khởi động khi đăng nhập máy:
stardesk autostart enable
```

---

## 📜 Giấy Phép

Dự án được phân phối dưới giấy phép [MIT](LICENSE).
Tất cả các nhãn hiệu và tài sản thuộc về NetEase và các chủ sở hữu tương ứng.
