# Chainz
No bullshit minimal Rust natve scheduler.

Built to support basic ETL and scheduling needs.

## Requirements

- Run time addition/removal of tasks
- Direct and dynamic binding to rust (loadlib?) or just using cargo projects
- Otherwise compilable rust
- Shell
- Super simple interface, "every(d, h, m, s)" or cron.
- Async
- Basic pushing of chained tasks

### Stretch
- Python bindings

## Architecture

Not sure how to handle rust code? Either add compiled binary to a folder and alter a yaml config file to add or remove tasks. Or add as cargo projects. Could probably do both, ie. pull from github/add raw cargo project then it is compiled and added to tasks as desired? Then won't need to look into using dynlib, although would be cool.

### Folder structure of execution environment

| exec root
|- scheduler.exe
|- tasks.yaml
--- tasks
    |- task2.exe
    |- task1.exe
    |- task3.exe
--- cargo projects
    |- crate1
    |- crate2   

### Using dynlib

Have main function that encapsulates main loop of program, then a dynlig linked library using libloader and cargo watch. Tasks are created in the library, problem is all tasks would need to be compiled when one is added. Then solution is recursive cargo projects with libloader, not ideal.