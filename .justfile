bash:
  echo "just > bash"

test:
  cargo geng build --platform web --package yasdk --example yatest
  pushd target/geng && zip -r ../ya.zip .

gh-pages:
  cargo geng build --platform web --release --out-dir gh-pages/web
  pushd gh-pages && git add . && git commit -m "YAY" && git push && popd

test-pages:
  cargo geng build --platform web --release --out-dir gh-pages/web
  caddy file-server --listen 127.0.0.1:8080 --browse --root gh-pages

deploy:
  cargo geng build --platform web --release
  butler push target/geng kuviman/meowjelly:html5
