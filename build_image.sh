set -e

make -C ../linux-e1000/ M=$PWD LLVM=1
cp ./r4l_e1000_demo.ko ../rootfs

pushd ../rootfs && \
find . | cpio -o --format=newc > ../rootfs.img && \
popd


qemu-system-x86_64 \
-netdev "user,id=eth0" \
-device "e1000,netdev=eth0" \
-object "filter-dump,id=eth0,netdev=eth0,file=dump.dat" \
-kernel ../linux-e1000/arch/x86/boot/bzImage \
-append "root=/dev/ram rdinit=sbin/init ip=10.0.2.15::10.0.2.1:255.255.255.0 console=ttyS0" \
-nographic \
-initrd ../rootfs.img


# -kernel ../linux-e1000/arch/x86/boot/bzImage \