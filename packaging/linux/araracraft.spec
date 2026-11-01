# Packages an already built release binary; build it with packaging/linux/build-rpm.sh.
%global debug_package %{nil}
%global __strip /bin/true
%define _build_id_links none

Name:           araracraft
Version:        %{game_version}
Release:        %{game_release}%{?dist}
Summary:        A block-building sandbox game written in Rust
License:        (MIT OR Apache-2.0) AND CC-BY-4.0 AND CC-BY-3.0 AND CC-BY-SA-3.0 AND CC-BY-SA-4.0 AND CC0-1.0
URL:            https://github.com/Githyot/AraraCraft
Obsoletes:      mine-clone < 0.2
Provides:       mine-clone = %{version}-%{release}
ExclusiveArch:  x86_64

# Loaded at runtime with dlopen, so rpm cannot find them on its own.
Requires:       vulkan-loader
Requires:       libxkbcommon
Requires:       libxkbcommon-x11
Requires:       libX11
Requires:       libXcursor
Requires:       libXi
Requires:       libXrandr
Requires:       libwayland-client
Requires:       libwayland-cursor
Recommends:     mesa-vulkan-drivers

%description
AraraCraft is a Minecraft-like block game written in Rust and rendered with
wgpu on Vulkan. Run it from the applications menu or with "araracraft".
Saves, settings and resource packs live in ~/.local/share/araracraft.

%install
mkdir -p %{buildroot}/opt/araracraft %{buildroot}%{_bindir} %{buildroot}%{_datadir}/applications
cp -a %{stage}/opt/araracraft/. %{buildroot}/opt/araracraft/
install -m 0755 %{stage}/araracraft.sh %{buildroot}%{_bindir}/araracraft
install -m 0644 %{stage}/araracraft.desktop %{buildroot}%{_datadir}/applications/araracraft.desktop
for s in 32 48 64 128 256 512; do
    install -D -m 0644 %{stage}/icons/icon_$s.png %{buildroot}%{_datadir}/icons/hicolor/${s}x${s}/apps/araracraft.png
done

%files
/opt/araracraft
%{_bindir}/araracraft
%{_datadir}/applications/araracraft.desktop
%{_datadir}/icons/hicolor/*/apps/araracraft.png
