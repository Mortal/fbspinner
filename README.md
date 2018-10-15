fbspinner
=========

Show a spinner on the framebuffer during system boot.

When combined with a [silent boot](https://wiki.archlinux.org/index.php/Silent_boot),
the resulting boot experience mimics that of another popular desktop OS.

[Video demonstration](https://www.youtube.com/watch?v=kdrZiFAybuc)

Installation
------------

Since fbspinner is written in Rust, you need to install [Rust and Cargo](https://www.rust-lang.org/install.html).

Your kernel should be compiled with `CONFIG_FRAMEBUFFER_CONSOLE_DEFERRED_TAKEOVER=y` to retain the vendor logo during kernel boot.
Check this with `zgrep FRAMEBUFFER_CONSOLE /proc/config.gz`.
Note that the config option was added recently in Linux 4.19 (released on October 21, 2018);
if your kernel is out of date or does not have this option, you should consider compiling a kernel with the option.

1. Build fbspinner with `cargo build --release`

2. `sudo mkdir -p /usr/local/share/fbspinner && sudo cp anim.bin /usr/local/share/fbspinner`

3. `sudo cp fbspinner.service fbspinner-quit.service /etc/systemd/system && sudo systemctl daemon-reload`

4. `sudo systemctl enable fbspinner`

5. Append to your kernel command line: `quiet loglevel=2 rd.udev.log_priority=2 rd.systemd.show_status=false systemd.show_status=false splash`

   If you use Intel graphics, also append `i915.fastboot=1`.

6. Reboot!
