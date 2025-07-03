dev:
    trap 'kill -9 $twpid | true; exit' EXIT
    ./tailwindcss -i src-styles/main.scss -o static/style.css --watch=always \
        & twpid=$!
    echo "Tailwind: watching"

    zola serve

build:
    ./tailwindcss -i src-styles/main.scss -o static/style.css --minify
    zola build

    mkdir -p "./build/www"
    cp -R public "./build/www"

    echo "Build: Done"
