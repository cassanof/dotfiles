# A cheatsheet guide for installing Arch Linux (after archiso boot)
## brought to you by Federico "Elleven11" Cassano.  

## - Step 0:
if you don't already have a arch live usb, or a prepared VM, this is not the guide for you. This guide is meant for people that already have a bootable live arch image.

In this guide i will give you some hacky shortcuts that will cut your time spent by half.

## - Step 0.5 Different keyboard layout
if your keyboard layout is not the US ANSI layout, type this command ```loadkeys layout``` where "layout" is the keyword for the layout you want  
Layouts available: ```localectl list-keymaps | less```

## - Step 1a Wired Connection:
When first booting into the live arch iso, you should check if your connection is working by pinging a website: ```ping gnu.org``` , if that does not work, try ```ping 1.1.1.1```. if the second option worked and the first did not: check /etc/resolv.conf and insert a public dns server, (if resolv.conf is a symlink created by NetworkManager; do ```rm /etc/resolv.conf``` before editing the existing one) e.g. resolv.conf file:
```
# cloudflare dns servers
nameserver 1.1.1.1
nameserver 1.0.0.1
```
Meanwhile if both ping commands did not work try doing the command: ```dhcpcd``` to create a ipv4/ipv6 local ip.
If still your connection does not work, consult the arch wiki page: https://wiki.archlinux.org/index.php/Network_configuration  

## - Step 1b Wi-Fi Connection:
if you are using a wireless connection, arch I you covered.  
Do ```ifconfig``` and memorize the name of the wireless interface (usualy its wlan0)  
Then check if rfkill is blocking the interface: ```rfkill list``` , if it is do: ```rfkill unblock interfacename``` , ofcourse, replacing "interfacename" with the name of your wireless interface.  
After that, do ```wifi-menu``` to connect to your wireless network, if the command is not present you need to install with a Wired Connection some packages:
- First do ```pacman -Syu``` to update the repositories
- Then do ```pacman -S dialog```    
  
and you should be able to use the command now.  

if you don't have access to a wired connection at all and ```wifi-menu``` was not built in into the current arch-iso, there are a lot of ways to manually connect, i prefer using wpa_supplicant:
- Edit ```/etc/wpa_supplicant/wpa_supplicant-interfacename```, most likely there wont be a file already, and ofcourse, change interfacename. e.g. config:
  ```
  ctrl_interface=/run/wpa_supplicant
  update_config=1

  network={
    ssid="MYSSID"
    psk="MYPASSWORD"
  }
  ```
- And now connect to the network by using: ```wpa_supplicant -B -i interfacename -c /etc/wpa_supplicant/wpa_supplicant-interfacename```
- For more info: https://wiki.archlinux.org/index.php/Wpa_supplicant#Configuration

After you connected to your network, repeat the steps for the Wired Connection Step.

## - Step 2a Disk Partitioning On Hardware
### Use this part of the guide only if you are installing on new hardware, if you are installing on old non-EFI hardware or a virtual machine use the other part of the guide below

Now, the hard part of the installation begins. In this guide i will be setting up 3 partitions on a drive:
- A boot partition (256M up to 512M)
- A swap partition (2G up to 8G)
- The root partition (The rest of the drive)

To start partitioning, we need to find the desired disk letter (```/dev/sdX``` , where X being the letter of the drive).  
If you only have 1 drive on your computer, the drive should be ```/dev/sda```  
To do that we can use two commands, ```lsblk``` and ```fdisk -l | less```  
After finding the drive we want to install arch on, use this command to wipe everything off the drive:
### WARNING THIS WILL DELETE EVERYTHING OFF THE DRIVE
```
wipefs --all /dev/sdX
```
Done that, we will start partitioning by doing ```cfdisk /dev/sdX``` (remember to replace X with the right drive letter, i will mention that a lot in this guide)  
After you see the terminal GUI open up, you will see that it asks you what label you want for the disk. This part of guide is for UEFI/EFI bios systems, for older hardware chose the DOS label and follow the other part of the partitioning guide  

So, choose ```GPT``` as the label if your BIOS is UEFI/EFI compatible (should be if its not over 10 years old)  
Now:
- To move around the GUI you can use the arrow keys, select the ```Free space``` device and select ```[ New ]```
 
- Make the first partition 512M by typing it in (you can make it smaller, but for good practice ill be doing that). Then, staying with the cursor on the ```/dev/sdX1/``` partition, use horizontal arrow keys to select ```[ Type ]``` and hit Enter. Go up and select ```EFI System```.

- Move the cursor to ```Free space``` again, and create a 2G, 4G or 8G partition, i will do 2G. Then change the type of that partition to ```Linux swap```

- Create the last partition with the rest of the space, ensuring that the type is ```Linux filesystem```

- Move the cursor horizontally to the ```[ Write ]``` option and hit Enter, write ```yes``` to confirm, and hit the key ```q``` to quit.

If everything went correctly, typing the command ```lsblk``` will show a drive (along side other drives) with partitions:
- sdX1   512M
- sdX2   2G to 8G
- sdX3 Rest of the drive

Now, we have to make the filesystem for these partitions:
- Do: ```mkfs.vfat /dev/sdX1``` (Always remember to replace the X with your drive letter)
- Then: ```mkswap /dev/sdX2``` and ```swapon /dev/sdX2```
- Lastly: ```mkfs.ext4 /dev/sdX3```

Now, if you want to check if you did filesystem correctly type ```lsblk -f```  
You should see:
- sdX1 vfat FAT32
- sdX2 swap 1 (if different version wont matter)
- sdX3 ext4 1.0 (if different version wont matter)

## - Step 2b Disk Partitioning On VM or Old Harware
### If you are installing Arch on old hardware or a Virtual Machine, then follow this part of the guide.
Do the steps in **2a** to identify the drive letter, if you are using a VM or you have only 1 drive, it should be ```/dev/sda/```  

Wipe everything off the drive with
```
wipefs --all /dev/sdX
```
where ```X``` in ```/dev/sdX``` is the letter of the drive you identified (i will be mentioning /dev/sdX a a lot in this guide so keep that in mind)

Then, start partitioning the drive with ```cfdisk /dev/sdX```  
Select DOS label and follow these steps:
- To move around the GUI you can use the arrow keys, select the ```Free space``` device and select ```[ New ]```

- Make the first partition 512M by typing it in (you can make it smaller, but for good practice ill be doing that). Then, staying with the cursor on the ```/dev/sdX1/``` partition, use horizontal arrow keys to select ```[ Bootable ]``` and hit Enter.

- Move the cursor to ```Free space``` again, and create a 2G, 4G or 8G partition, i will do 2G. Then use horizontal arrows to select ```[ Type ]```, select the ```Linux swap / Solaris``` type.

- Create the last partition with the rest of the space.

- Move the cursor horizontally to the ```[ Write ]``` option and hit Enter, write ```yes``` to confirm, and hit the key ```q``` to quit.

If everything went correctly, typing the command ```lsblk``` will show a drive (along side other drives) with partitions:
- sdX1   512M
- sdX2   2G to 8G
- sdX3 Rest of the drive

Now, we have to make the filesystem for these partitions:
- Do: ```mkfs.ext4 /dev/sdX1``` (Always remember to replace the X with your drive letter)
- Then: ```mkswap /dev/sdX2``` and ```swapon /dev/sdX2```
- Lastly: ```mkfs.ext4 /dev/sdX3```

Now, if you want to check if you did filesystem correctly type ```lsblk -f```  
You should see:
- sdX1 ext4
- sdX2 swap
- sdX3 ext4

## - Step 2.5 Mounting the partitions
Lets mount the drives now, type ```mount /dev/sdX3 /mnt```, then ```mkdir /mnt/boot``` and ```mount /dev/sdX1 /mnt/boot```  
Replace X in ```/dev/sdX``` with your drive letter

## - Step 3 Pacstrapping (a little trick)
Run ```pacstrap /mnt base base-devel linux linux-firmware vim vi nano```If you are having problems with signatures, you can remove them by editing ```/etc/pacman.conf``` and changing the ```SigLevel``` line to: ```SigLevel = Never```

And now, you have successfully installed a whole linux system into your drive, but we still have some stuff to do before we can get it up and running.

## - Step 4 Generating fstab
The fstab (File System Table) file, is the file that controls how partitions are mounted at boot, because if we mount partitions with the ```mount``` command, when we reboot the partitions won't be mounted anymore.  
Arch has an amazing script to generate automatically the fstab file, do this command to do so: ```genfstab -U /mnt >> /mnt/etc/fstab```  
To see if it has done it correctly type: ```sed "" /mnt/etc/fstab```  
It should print out the UUID of the drivers with the mounting point  
For more info and troubleshooting: https://wiki.archlinux.org/index.php/Fstab

## - Step 5 Chroot and boot setup
Chroot means Change Root, it will enter the new installed system. Command: ```arch-chroot /mnt```

After you chrooted into your new environment, we should start to setup pacman, the arch package manager.  
First of all, initialize a signature keyring, this will keep all trustable signatures from arch developers:  
```pacman-key --init``` and ```pacman-key --populate```  
Then, we should update the system and its repositories with the command ```pacman -Syu``` (for more info on pacman options: https://wiki.archlinux.org/index.php/Pacman)

Then, we should install a network manager (otherwise, you wond have internet on boot) and a bootloader. For the network, if you are using Wi-Fi, i suggest to use netctl instead of NetworkManager. Meanwhile, for the bootloader, you should always use GRUB, its the less bloated and if you have a problem with it you can usually search the solution on the wiki.
```
pacman -S networkmanager grub
```
And now, enable NetworkManager on boot: ```systemctl enable NetworkManager```

Then, comes a part where a lot of people fail, the grub install.
If you have partitioned the drive with the label GPT, you have followed correctly my steps in **2a** and you are not on a Virtual Machine do these commands:
```
pacman -S efibootmgr

grub-install --target=x86_64-efi --efi-directory=/boot /dev/sdX
```
Replacing the X in ```/dev/sdX``` with your drive letter
  
If you are using a VM, or your system is not EFI compatible and you have followed correctly my steps in **2b** do this command instead:
```
grub-install --target=i386-pc /dev/sdX
```
Replacing the X in ```/dev/sdX``` with your drive letter

If the grub installation finished without problems, generate a grub.cfg with the command ```grub-mkconfig -o /boot/grub/grub.cfg```

If you had any errors popup here is the link to the arch wiki: https://wiki.archlinux.org/index.php/GRUB  

## - Step 6 Language and Timezone
Use the editor of your choice (I use vim) to open ```/etc/locale.gen``` and uncomment the language of your choice (removing the # at the start of the line)

you should uncomment both UTF-8 and ISO languages, e.g. for en_US:
```
#en_SG ISO-8859-1
en_US.UTF-8 UTF-8
en_US ISO-8859-1
#en_ZA.UTF-8 UTF-8
#en_ZA ISO-8859-1
```
I found en_US at the 176th line.  
Then, do ```locale-gen``` to generate the locales  
Now, create ```/etc/locale.conf``` and select your language using this syntax:
```
LANG=en_US.UTF-8
```
Feel free to change ```en_US.UTF-8``` to your own language  

Now, you need to setup the timezone. First of all do the command: ```timedatectl set-ntp true```, then use the command ```date``` to check if the time is correct, if its not, do this command to list all timezones:  
```timedatectl list-timezones``` (press q to quit)  
and then select a timezone with:  
```timedatectl set-timezone Region/City```

Check again with ```date``` to see if the time is correct.

## - Step 7 User Setup
Create/Edit the file ```/etc/hostname``` and write in the hostname you want for your system, e.g. ```archbox```  

Now, change the root password by typing: ```passwd root```

Then, you should create a secondary user, to do that type:  
```useradd -mg wheel demo``` replacing "demo" with the desired username, and change the password of that user by doing: ```passwd demo```  

Then, you should give the ```wheel``` group sudo permissions. To do that use the ```visudo``` command, it will enter in a vi/vim session.

If you are not familiar with vi/vim, you first need to scroll down to the line where it says:
```
## Uncomment to allow members of group wheel to execute any command
# %wheel ALL=(ALL) ALL
```
Then you need to place the cursor on the ```#``` and press ```x``` to delete that character.
Then, write ```:wq``` to save and quit vi/vim

## - Step 8 Un-mount Drives and Quit
We are almost done! last things we have to do is un-mount the drives and reboot.

To do so, start exiting chroot by typing in ```exit``` , you will be prompted back to the "red and white" zsh shell.  
Now, type ```umount -R /mnt``` 
Then, shutdown the system by either writing ```shutdown now``` or by pressing the power button, when the system power is off, remove the usb drive/disk with the archiso and power the system back on.

If the installation went correctly you should see the grub bootloader screen, and soon you will be asked to insert login credentials.  

## You just installed Arch Linux!
