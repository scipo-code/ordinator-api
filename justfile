build-windows:
    cross build --target x86_64-pc-windows-gnu

build-linux:
    cargo build 

tr REGEX:
    tail -F logging/logs/ordinator.developer.log | rg {{ REGEX }}
    
