dev:
    trap 'kill -9 $twpid | true; exit' EXIT
    ./tailwindcss -i src-styles/main.scss -o static/style.css --watch=always \
        & twpid=$!
    echo "Tailwind: watching"

    zola serve

build:
    echo "pwd: `pwd`"
    ls -la

    cp /usr/local/bin/tailwindcss ./
    ./tailwindcss -i src-styles/main.scss -o static/style.css --minify
    zola build

    mkdir -p "./build/www"
    cp -R public "./build/www"
    cp -R resources "./build/www/public"

    ls -la "./build/www/public"

    echo "Build: Done"
