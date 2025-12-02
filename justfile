dev:
    tmux new-session -d -s top-doggo \; send-keys 'vim .' Enter \; new-window \; \
    send-keys 'nix develop -c just w' Enter \; new-window \; \
    send-keys 'git pull' Enter \; new-window \; \
    send-keys 'nix develop -c just db' Enter \; new-window \; \
    send-keys 'just tww' Enter \; \
    attach-session -t top-doggo


tailwind-compile:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css
tw: tailwind-compile

tailwind-compile-watch:
    npx tailwindcss -i ./assets/input.css -o ./assets/output.css --watch
tww: tailwind-compile-watch

watch:
    cargo watch -x run
w: watch

fix:
    cargo watch -x fix

db:
    sqlite3 db/top-doggo.db

clippy:
    cargo clippy --fix --allow-dirty
remove-imports: clippy


# docker image ls # to determine the last version tag used
# docker build -t parkerbedlan/top-doggo:0.0.19 .
# docker run -p 3002:3000 -v ./db:/db parkerbedlan/top-doggo:0.0.19
# docker tag parkerbedlan/top-doggo:0.0.19 ghcr.io/parkerbedlan/top-doggo:0.0.19
# docker push ghcr.io/parkerbedlan/top-doggo:0.0.19

dbuild version:
    docker build -t parkerbedlan/top-doggo:{{version}} . && just drun {{version}}

drun version:
    docker run -p 3002:3000 -v ./db:/db -v ./assets/images:/assets/images -v ./unapproved:/unapproved parkerbedlan/top-doggo:{{version}}
   
dpush version:
    docker tag parkerbedlan/top-doggo:{{version}} ghcr.io/parkerbedlan/top-doggo:{{version}} && docker push ghcr.io/parkerbedlan/top-doggo:{{version}}

dfull version:
    docker build -t parkerbedlan/top-doggo:{{version}} . && docker tag parkerbedlan/top-doggo:{{version}} ghcr.io/parkerbedlan/top-doggo:{{version}} && docker push ghcr.io/parkerbedlan/top-doggo:{{version}}

ssh:
    ssh root@5.161.95.82

ssh-db:
    scp root@5.161.95.82:/root/top-doggo/db/* . && sqlite3 top-doggo.db && rm top-doggo.db*

nd:
    nix develop
