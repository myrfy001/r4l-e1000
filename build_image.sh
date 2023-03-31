set -e

# Build the kernel module
make -C ../linux-e1000/ M=$PWD LLVM=1 CLIPPY=1

# Copy the build kernel module to the rootfs folder, there should be a usable busybox at `rootfs` folder
cp ./r4l_e1000_demo.ko ../rootfs

# Build the rootfs image
pushd ../rootfs && \
find . | cpio -o --format=newc > ../rootfs.img && \
popd


# Run QEMU to test the kernel, the network traffic will be dumped to `dump.dat` and you can use wireshark to explore the result.
qemu-system-x86_64 \
-netdev "user,id=eth0" \
-device "e1000,netdev=eth0" \
-object "filter-dump,id=eth0,netdev=eth0,file=dump.dat" \
-kernel ../linux-e1000/arch/x86/boot/bzImage \
-append "root=/dev/ram rdinit=sbin/init ip=10.0.2.15::10.0.2.1:255.255.255.0 console=ttyS0" \
-nographic \
-initrd ../rootfs.img
