{ team, config, pkgs, lib, ... }:

let 
  inherit (team) drone wifi;
in 
{
  environment.noXlibs = lib.mkForce false;

  fileSystems."/" =
    { device = "/dev/disk/by-label/NIXOS_SD";
      fsType = "ext4";
    };

  boot = {
    kernelPackages = lib.mkForce pkgs.linuxPackages_latest;
    loader = {
      generic-extlinux-compatible.enable = lib.mkDefault true;
      grub.enable = lib.mkDefault false;
    };
  };

  nix.settings = {
    experimental-features = lib.mkDefault "nix-command flakes";
    trusted-users = [ "root" "@wheel" ];
  };


  environment.systemPackages = with pkgs; 
    [ gcc unzip rustc cargo vim curl wget nano bind iptables i2c-tools openvpn python3 rsync tmux helix python311Packages.rpi-gpio];

  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "yes";
      PasswordAuthentication = true;
    };
  };

  programs.zsh.enable = true;
  virtualisation.docker.enable = true;
  networking.firewall.enable = false;

  hardware = {
    enableRedistributableFirmware = true;
    firmware = [ pkgs.wireless-regdb ];
    raspberry-pi."4".i2c1.enable = true;
  };

  networking = {
    hostName = team.name;

    interfaces.wlan0.useDHCP = true;
    interfaces.wlp1s0u1u2.useDHCP = true;
    interfaces.eth0.useDHCP = true;

    wireless.enable = true;
    wireless.interfaces = [ "wlan0" ];
    wireless.networks.${wifi.ssid}.psk = wifi.passphrase;
  };

  systemd.services.aviator = {
    description = "Controls the drones aviation status";
    wantedBy = [ "default.target" ];

    startLimitBurst = 10;

    serviceConfig = {
      User = "root";
      Group = "root";
      ExecStart = "${./aviator}";
      Restart = "always";
      RestartSec = "5";
    };
  };

  systemd.services.drone-wifi = {
    description = "Connects to the drone's wifi using wpa_supplicant";
    wantedBy = [ "default.target" ];

    startLimitBurst = 10;

    serviceConfig = {
      User = "root";
      Group = "root";
      ExecStart = 
        let script = pkgs.writeShellApplication {
          name = "drone-wifi";
          runtimeInputs = with pkgs; [ sudo coreutils wpa_supplicant ];
          text = ''
            config=$(mktemp)
            wpa_passphrase ${drone.ssid} ${drone.passphrase} > "$config"
            echo ${team.key} | sudo -S wpa_supplicant -c "$config" -i wlp1s0u1u2
          '';
        }; in "${script}/bin/drone-wifi";
      Restart = "always";
      RestartSec = "5";
    };
  };

  users.defaultUserShell = pkgs.zsh;
  users.mutableUsers = true;
  users.groups.hack = { gid = 1000; name = "hack"; };

  users.users = {
    hack = {
      uid = 1000;
      home = "/home/hack";
      name = "hack";
      group = "hack";
      shell = pkgs.zsh;
      extraGroups = [ "wheel" "docker" "i2c" ];
      initialPassword = team.key;
      isNormalUser = true;
    };

    root = {
      shell = pkgs.zsh;
      extraGroups = [ "wheel" "docker" "i2c" ];
      initialPassword = team.key;
    };
  };

  system.stateVersion = "24.05";
}
