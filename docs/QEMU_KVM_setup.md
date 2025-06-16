# KVM

## What KVM will be used for?

Doc needed here


## How to get KVM on your computer
The following tutorial assumes that you use a Debian or Ubuntu distribution.

First, you need to check if you CPU supports virtualization
```bash 
egrep -c '(vmx|svm)' /proc/cpuinfo
```
The result of the command should be an integer greater than 0

In addition you can verify in KVM virtualization is enabled by using the `kvm-ok` cmd. First you need to install `cpu-checker` utils:
```bash 
sudo apt install cpu-checker
kvm-ok
```
This should return something like 
```bash
INFO: /dev/kvm exists
KVM acceleration can be used
```

Install KVM on your computer:
```bash
sudo apt install -y qemu-kvm virt-manager libvirt-daemon-system virtinst libvirt-clients bridge-utils
```

Start and enable Virtualization Daemon
```bash
sudo systemctl enable --now libvirtd
sudo systemctl start libvirtd
```

Confirm the daemon is running
```bash
sudo systemctl status libvird
```
Add the current logged-in user in the kvm and libvirt groups to be able to create and manage virtual machines
```bash
sudo usermod -aG kvm $USER
sudo usermod -aG libvirt $USER
```