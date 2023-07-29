# MiniCompiler

参考资料：https://pku-minic.github.io/online-doc/#/

编译指令：`cargo run -- -koopa hello.c -o hello.koopa`

启动docker指令：` docker run -it --rm -v <project path>:/root/compiler maxxing/compiler-dev bash`

lv1测试：`docker run -it --rm -v 项目目录:/root/compiler maxxing/compiler-dev autotest -koopa -s lv1 /root/compiler`

测试数据再`/opt/bin/testcases`目录下