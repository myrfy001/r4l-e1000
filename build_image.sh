set -e

make -C ../linux-e1000/ M=$PWD LLVM=1
cp ./r4l_e1000_demo.ko ../rootfs

cd ../rootfs && \
find . | cpio -o --format=newc > ../rootfs.img

qemu-system-x86_64 \
-kernel ./arch/x86/boot/bzImage \
-append "root=/dev/ram rdinit=sbin/init ip=10.0.2.15::10.0.2.1:255.255.255.0 console=ttyS0" \
-nographic \
-initrd ../rootfs.img