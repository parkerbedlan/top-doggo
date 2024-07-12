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
    sqlite3 db/todos.db

#tmux new-session -d -s best-doggo \; send-keys 'vim .' Enter \; new-window \; send-keys 'just w' Enter \; new-window \; send-keys 'git pull' Enter \; new-window \; send-keys 'just tww' Enter \; attach-session -t best-doggo
dev:
    tmux new-session -d -s best-doggo \; send-keys 'vim .' Enter \; new-window \; send-keys 'just w' Enter \; new-window \; send-keys 'git pull' Enter \; new-window \; send-keys 'just db' Enter \; new-window \; send-keys 'just tww' Enter \; attach-session -t best-doggo

# docker image ls # to determine the last version tag used
# docker build -t parkerbedlan/best-doggo:0.0.19 .
# docker run -p 3002:3000 -v ./db:/db parkerbedlan/best-doggo:0.0.19
# docker tag parkerbedlan/best-doggo:0.0.19 ghcr.io/parkerbedlan/best-doggo:0.0.19
# docker push ghcr.io/parkerbedlan/best-doggo:0.0.19

dbuild version:
    echo "docker build -t parkerbedlan/best-doggo:{{version}} . && docker run -p 3002:3000 -v ./db:/db parkerbedlan/best-doggo:{{version}}"
   
dpush version:
    echo "docker tag parkerbedlan/best-doggo:{{version}} ghcr.io/parkerbedlan/best-doggo:{{version}} && docker push ghcr.io/parkerbedlan/best-doggo:{{version}}"
