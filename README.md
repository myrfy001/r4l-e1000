## This is a project trying to port Intel E1000 driver to Rust For Linux in 7 days.

This project is built from scratch, starting from the R4L out of tree template(https://github.com/Rust-for-Linux/rust-out-of-tree-module), with some reference form:

* https://github.com/Rust-for-Linux/linux/blob/rust/drivers/net/ethernet/intel/e1000/e1000_main.c
* https://github.com/elliott10/e1000-driver
* https://github.com/fujita/rust-e1000
* https://pdos.csail.mit.edu/6.828/2011/labs/lab6/

Since this project is written from scratch, you can view the commit log for the develope process. And there is also a development log available. I will release it if possible.

----


The kernel that the module is built against needs to be Rust-enabled (`CONFIG_RUST=y`).

  - The kernel tree (`KDIR`) requires the Rust metadata to be available. These are generated during the kernel build, but may not be available for installed/distributed kernels (the scripts that install/distribute kernel headers etc. for the different package systems and Linux distributions are not updated to take into account Rust support yet).

  - All Rust symbols are `EXPORT_SYMBOL_GPL`.

Try running it with (you should read this shell script and modify some variables to match your environment):

```sh
$ bash ./build_image.sh
```

When the kernel started, type the following command to install the driver and config the network interface:

```sh
insmod r4l_e1000_demo.ko
ip link set eth0 up
ip addr add broadcast 10.0.2.255 dev eth0
ip addr add 10.0.2.15/255.255.255.0 dev eth0 
ip route add default via 10.0.2.1 
```

Then, ping the host to see the final result:

```sh
ping 10.0.2.2
```


For details about the Rust support, see https://rust-for-linux.com.

For details about out-of-tree modules, see https://docs.kernel.org/kbuild/modules.html.

## rust-analyzer

Rust for Linux (with https://lore.kernel.org/rust-for-linux/20230121052507.885734-1-varmavinaym@gmail.com/ applied) supports building a `rust-project.json` configuration for [`rust-analyzer`](https://rust-analyzer.github.io/), including for out-of-tree modules:

```sh
make -C .../linux-with-rust-support M=$PWD rust-analyzer
```
