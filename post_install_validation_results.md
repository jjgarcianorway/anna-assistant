# Post-Install Question Validation Results

**Date:** 2025-11-19 07:44:36
**Anna Version:** annactl 5.7.0-beta.80
**Questions File:** data/post_install_questions.json
**Questions Tested:** 10

---

## Test Results

### Question 1: My internet doesn't work after installation. How do I check if my network is connected?

**Category:** network | **Difficulty:** beginner

**Anna's Response:**
```


[2m[...] Working...[0m



 [1m[96mAnna[39m[0m
[2m────────────────────────────────────────────────────────────[0m

[36mℹ[39m [2mLet's start by checking your network connection. You can use the `ip link` command to see which interfaces (wired or wireless) are linked to an IP address.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mRun the following command in a terminal:[0m
[36mℹ[39m [2m```bash[0m
[36mℹ[39m [2mip link show[0m
[36mℹ[39m [2m```[0m
[36mℹ[39m [2mThis will list all your network interfaces. Look for the interface that is currently connected and note its name. This should be something like "wlp3s0" or "eth0".[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mIf you don't see any interfaces listed, it's possible that your network card is not detected. In this case, try installing the `ip` package:[0m
[36mℹ[39m [2m```bash[0m
[36mℹ[39m [2msudo pacman -S ip[0m
[36mℹ[39m [2m```[0m
[36mℹ[39m [2mOnce you have a connected interface, run:[0m
[36mℹ[39m [2m```bash[0m
[36mℹ[39m [2mip addr show[0m
[36mℹ[39m [2m```[0m
[36mℹ[39m [2mThis will list all IP addresses assigned to each interface.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mIf you're still having trouble connecting, we can troubleshoot further.[0m
```

**Validation:** ✅ PASSED

✓ Expected commands found: 2/3\n⚠ Expected topics not found: NetworkManager, systemd-networkd, dhcpcd\n

### Question 2: How do I connect to my WiFi network?

**Category:** network | **Difficulty:** beginner

**Anna's Response:**
```


[2m[...] Working...[0m



 [1m[96mAnna[39m[0m
[2m────────────────────────────────────────────────────────────[0m

[36mℹ[39m [2mTo connect to your WiFi network, make sure the wireless interface (usually `wlan0` or `wlp3s0`) is enabled. You can check if it's enabled by running the command `ip link`.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mIf it's not enabled, you can enable it with the command `ip link set dev wlan0 up`. [0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mTo connect to your WiFi network, use the `nmcli` command:[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2m```[0m
[36mℹ[39m [2mnmcli d wifi connect <your_networkSSID> password <your_wifi_password>[0m
[36mℹ[39m [2m```[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mReplace `<your_networkSSID>` with the name of your Wi-Fi network and `<your_wifi_password>` with your Wi-Fi password. You can find your network's SSID in your router's settings.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mAlternatively, you can use `netctl` or `netcfg` commands if you prefer. The Arch Wiki has more information on these topics: https://wiki.archlinux.org/title/Network_configuration[0m
```

**Validation:** ✅ PASSED

✓ Expected commands found: 1/3\n⚠ Expected topics not found: iwd, NetworkManager, WPA supplicant\n

### Question 3: How do I install a program on Arch Linux?

**Category:** packages | **Difficulty:** beginner

**Anna's Response:**
```


[2m[...] Working...[0m



 [1m[96mAnna[39m[0m
[2m────────────────────────────────────────────────────────────[0m

[36mℹ[39m [2mTo install a program on Arch Linux, you can use the `pacman` package manager. Here are a few ways to install software:[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2m1. **Using the terminal**: Search for the package name in the terminal using the `pacman -Ss` command, and then install it with `pacman -S <package_name>`.[0m
[36mℹ[39m [2m2. **Using Arch User Repository (AUR)**: If you want to install a package not available in the official repositories, you can use the AUR. Search for the package name on the AUR website or using the `aur-complete` command in the terminal.[0m
[36mℹ[39m [2m3. **Using GUI tools**: You can also use graphical user interface tools like [Pamac](https://wiki.archlinux.org/title/Pacman) or [GNOME Software](https://wiki.gnome.org/Projects/School/SoftwareCenter) to install software.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mRemember to always verify the package's integrity by checking its GPG signature before installation.[0m
```

**Validation:** ✅ PASSED

✓ Expected commands found: 2/2\n✓ Expected topics found: 3/3\n

### Question 4: What's the difference between pacman -S and pacman -Sy?

**Category:** packages | **Difficulty:** beginner

**Anna's Response:**
```


[2m[...] Working...[0m



 [1m[96mAnna[39m[0m
[2m────────────────────────────────────────────────────────────[0m

[36mℹ[39m [2m`pacman -S` updates installed packages to their latest versions, whereas `pacman -Sy` updates both the package database (which lists all available packages) and installs packages that are missing on your system.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mIn other words, `-S` focuses on updating existing packages, while `-Sy` includes an additional step of synchronizing the package list with the repositories.[0m
```

**Validation:** ❌ FAILED

⚠ Expected topics not found: partial upgrade, sync database, best practices\n❌ Required warning missing: Never do partial upgrades\n

### Question 5: I want to install Firefox but pacman says 'package not found'. What's wrong?

**Category:** packages | **Difficulty:** intermediate

**Anna's Response:**
```

[2m[...] Working...[0m

[38;5;120m✓ No urgent issues right now![0m

Your system looks good. I'll let you know if I notice anything.
```

**Validation:** ✅ PASSED

⚠ Expected commands not found: pacman -Syu, pacman -Ss firefox\n⚠ Expected topics not found: database sync, repository mirrors\n

### Question 6: How do I install a desktop environment?

**Category:** display | **Difficulty:** beginner

**Anna's Response:**
```


[2m[...] Working...[0m



 [1m[96mAnna[39m[0m
[2m────────────────────────────────────────────────────────────[0m

[36mℹ[39m [2mTo install a desktop environment, you'll need to decide on one and then use pacman to install it. Some popular options include GNOME, KDE Plasma, XFCE, and i3.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mFirst, list available desktop environments with `pacman -Ss xdg-desktop-environment`. This will show you a list of packages that include various desktop environments.[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mOnce you've chosen an environment, run the command to install it: `sudo pacman -S <env_name>`, replacing `<env_name>` with the actual name (e.g., gnome or kde-plasma).[0m
[36mℹ[39m [2m[0m
[36mℹ[39m [2mAfter installation, you may need to log out and back in for the new desktop environment to take effect.[0m
```

**Validation:** ✅ PASSED

✓ Expected topics found: 3/5\n

### Question 7: My screen resolution is wrong. How do I fix it?

**Category:** display | **Difficulty:** intermediate

**Anna's Response:**
```

[2m[...] Working...[0m

[36mℹ[39m [2mLet me show you the available fixes...[0m



Which suggestion would you like me to apply?

  1. Clean up old package cache
  2. Install pacman-contrib for cache management
  0. Cancel

Enter number: 
```

**Validation:** ✅ PASSED

⚠ Expected commands not found: xrandr, lspci | grep VGA\n⚠ Expected topics not found: display drivers, NVIDIA, AMD\n


---

## Summary

| Metric | Count | Percentage |
|--------|-------|------------|
| **Total Questions** | 10 | 100% |
| **Passed** | 6 | 60.0% |
| **Failed** | 1 | 10.0% |
| **Warnings** | 7 | - |

### Assessment

❌ **NEEDS IMPROVEMENT:** Anna requires significant prompt or training improvements (<60% success rate).
