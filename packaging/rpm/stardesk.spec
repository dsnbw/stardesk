Name:           stardesk
Version:        1.5.4
Release:        1%{?dist}
Summary:        Pure Native Linux implementation of StarDesk Remote Desktop (No Wine)
License:        MIT
URL:            https://www.stardesk.com

Requires:       pipewire, wayland-libs, xdg-desktop-portal
Recommends:     libva-utils

%description
StarDesk is an ultra-low-latency remote desktop and game streaming software.
This package provides the 100% native Linux host daemon and viewer,
PipeWire desktop capture, /dev/uinput input injection, and hardware acceleration.
Zero Wine required.

%prep

%build

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_udevrulesdir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/256x256/apps

install -m 755 %{_sourcedir}/stardesk %{buildroot}%{_bindir}/stardesk
install -m 644 %{_sourcedir}/packaging/stardesk.desktop %{buildroot}%{_datadir}/applications/stardesk.desktop
install -m 644 %{_sourcedir}/assets/stardesk.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/stardesk.png
install -m 644 %{_sourcedir}/packaging/60-stardesk-uinput.rules %{buildroot}%{_udevrulesdir}/60-stardesk-uinput.rules

%post
/usr/bin/udevadm control --reload-rules 2>/dev/null || :
/usr/bin/udevadm trigger 2>/dev/null || :
/usr/bin/update-desktop-database &> /dev/null || :

%files
%{_bindir}/stardesk
%{_datadir}/applications/stardesk.desktop
%{_datadir}/icons/hicolor/256x256/apps/stardesk.png
%{_udevrulesdir}/60-stardesk-uinput.rules

%changelog
* Sun Nov 01 2026 StarDesk Linux Contributors <support@stardesk.com> - 1.5.4-1
- 100% Pure Native Linux implementation (Zero Wine)
