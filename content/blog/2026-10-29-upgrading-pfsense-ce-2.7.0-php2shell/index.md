+++
title = "Upgrading pfSense CE 2.7.0 to 2.8.1 (and surviving too small an EFI partition)"
# description = ""

[taxonomies]
tags = ["php2shell", "pfSense"] 
+++

Disclaimer: a stochastic model was used to parse terminal logs to prepare this content; I, as the human, have cleaned this up for readability. I, also as the human, ran these commands to upgrade and reboot my pfSense "PC in a box" router.

Goal: get an old pfSense CE box from **2.7.0-RELEASE** up to **2.8.1-RELEASE** to pick up the PHP 8.3.36 fix for the "PHP2Shell" vulnerabilities (CVE-2026-109618 / CVE-2026-109619). The GUI refused to switch the update branch, which kicked off a longer-than-expected journey.

---

## 1. The GUI wouldn't switch branches

System > Update > Update Settings _showed_ 2.8.1, but selecting it reverted to 2.7.0. The 2.7.0 build's repo definition was stale, so the dropdown wouldn't stick. Moved to the shell.

## 2. First shell attempt — privilege + OS-version warnings

Run as root, you can use `sudo -i` once you login to the pfSense router:

```shell
export TERM="xterm"
pkg-static install -fy pkg pfSense-repo pfSense-upgrade

# output:
pkg-static: Warning: Major OS version upgrade detected.  Running "pkg bootstrap -f" recommended
pkg-static: Insufficient privileges to install packages
```

## 3. Bootstrap pkg for the new major version

```shell
pkg-static bootstrap -f

# output:
Bootstrapping pkg from pkg+https://pkg.pfsense.org/pfSense_v2_8_1_amd64-pfSense_v2_8_1, please wait...
Newer FreeBSD version for package pkg:
- package: 1500029
- running kernel: 1400085
Ignore the mismatch and continue? [y/N]:
```

Answered **y** (expected at the FreeBSD 14 → 15 boundary).

## 4. Confirm the upgrade is now offered

```shell
pfSense-upgrade -C

# output:
2.8.1 version of pfSense is available
```

## 5. Run the upgrade — and hit the EFI wall

```shell
pfSense-upgrade

# output:
Updating boot code...
Only 636KB space remaining: removing old FreeBSD boot1.efi file /efi/boot/bootx64.efi
Failed to update the EFI System Partition /dev/ada0p1
Insufficient space remaining for /boot/loader.efi
Unable to update boot code on /dev/ada0
```

This is pfSense Bug #15081: the EFI **partition** is a normal size, but the FAT **filesystem** inside it was formatted tiny by an old installer. The new loader won't fit.

**Do not reboot here** — the old loader was already deleted and the new one didn't write. If I rebooted here, it wouldn't be able to boot back into pfSense.

## 6. Diagnose the ESP

```shell
mount -t msdosfs /dev/ada0p1 /mnt/esp
find /mnt/esp -type f
```

Only a 12-byte `EFI/BOOT/STARTUP.NSH` from 2016 — nothing to delete for space.

```shell
gpart show ada0

# output:
=>       40  468862048  ada0  GPT  (224G)
         40     409600     1  efi  (200M)      <- partition is 200M, good
     409640  460963840     2  freebsd-ufs  (220G)
  461373480    7488607     3  freebsd-swap  (3.6G)
```

Partition is 200M, so this is **recoverable without a reinstall** — just reformat the filesystem to use the whole partition.

```shell
ls -lah /boot/loader.efi
# -r-xr-xr-x  2 root  wheel   644K Aug 28  2025   (the 2.8.1 loader; 644K)
```

## 7. Reformat the ESP and restore the loader

FAT32 was rejected (partition too small for FAT32's cluster minimum):

```shell
newfs_msdos -F 32 -L EFISYS /dev/ada0p1
# newfs_msdos: 25573 clusters too few clusters for FAT32, need 65525
```

FAT16 is the correct choice for a 200M ESP:

```shell
newfs_msdos -F 16 -L EFISYS /dev/ada0p1
mount -t msdosfs /dev/ada0p1 /mnt/esp
mkdir -p /mnt/esp/EFI/BOOT /mnt/esp/EFI/freebsd
cp /boot/loader.efi /mnt/esp/EFI/BOOT/BOOTX64.EFI
cp /boot/loader.efi /mnt/esp/EFI/freebsd/loader.efi
df -h /mnt/esp
# /dev/ada0p1    200M    1.3M    199M     1%    /mnt/esp   <- fixed
umount /mnt/esp
```

Re-run the boot installer — clean this time:

```shell
/usr/local/sbin/../libexec/install-boot.sh -b auto -f ufs -s gpt -u ada0

# output:
Creating UEFI boot entry for FreeBSD
Marking UEFI boot entry 0000 active
Finished updating ESP
```

Rebooted.

## 8. Post-reboot: half-upgraded state

```shell
cat /etc/version   # 2.7.0-RELEASE      <- userland still old
uname -r           # 15.0-CURRENT       <- new kernel booted
php -v             # PHP 8.2.6          <- still old
```

The interrupted run replaced the kernel but not the userland. Needed to finish the upgrade.

## 9. Repo fetch fails (looked like DNS, was actually TLS)

```shell
pfSense-upgrade

# output:
pkg-static: An error occured while fetching package
Unable to update repository pfSense-core
Error updating repositories!
```

DNS check:

```shell
ping -c3 pkg.pfsense.org   # ping: Unknown host
ping -c3 1.1.1.1           # works -> routing fine, name resolution broken
```

Local Unbound (with DNSSEC) had died during the partial upgrade and was failing closed. Temporary workaround to get resolving again, then DNSSEC confirmed working:

```shell
drill -D pfsense.org       # works
```

But the repo update still failed. `/etc/resolv.conf` actually had plenty of working resolvers, so this wasn't DNS. The real cause was a stale CA trust store after the partial upgrade — fixed with:

```shell
certctl rehash
pkg-static update -f       # worked
```

## 10. Finish the upgrade

```shell
pfSense-upgrade

# output:
202064KB space remaining on ESP: renaming old loader.efi ...
Existing UEFI FreeBSD boot entry found: not creating a new one
Finished updating ESP
Done.  System is going to be upgraded.  Rebooting in 10 seconds.
```

---

## Lessons learned

- **The GUI branch selector on 2.7.0 is buggy** — do the branch switch from the CLI.
- **Old ESPs are a landmine.** A 200M EFI partition can still hold a ~700KB FAT filesystem. Reformat it to FAT16 and the loader fits; no reinstall needed if the _partition_ is large enough. (If the partition itself is ~800K, you must reinstall.)
- **Never reboot after "Unable to update boot code"** until you've written a valid loader — the old one may already be gone.
- **"error fetching package" can be TLS, not DNS.** `certctl rehash` after a partial upgrade fixes a stale CA bundle that looks exactly like a DNS failure.
- **DNSSEC/Unbound fails closed** — if it dies mid-upgrade, the box resolves nothing until you bypass it or it's restarted on a healthy base.
- Keep **console/IPMI** access ready for any FreeBSD-major upgrade.
