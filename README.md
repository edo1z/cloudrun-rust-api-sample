### main.rsを作成
- 0.0.0.0:8080で受け付けるWebServerを作成。
  - 本当は`PORT`という環境変数に応じて`8080`を変更するらしいけど、とりあえず、`8080`固定にしてみた。
  - `127.0.0.1`とか`localhost`とかだとダメみたいな記載もどこかにあったけど、しっかり試してない。

```main.rs
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

### Dockerfileを作成

```
FROM rust:latest
WORKDIR /pj
COPY . .
RUN cargo build --release
ENV PORT 8080
EXPOSE 8080
ENTRYPOINT [ "target/release/cloudrun-1" ]
```

- ENV PORTの設定がないとエラーになるかも。（しっかり確認してない）

### ビルド（イメージの作成）

```shell
> docker build -t asia.gcr.io/${PROJECT-ID}/cloudrun-1 .
```

#### 注意
- `asia.gcr.io` は、Container Registryで設定しているドメインによって変える。
- `${PROJECT-ID}`は、自分のプロジェクトIDに変える。
- `cloudrun-1`は、自分のアプリ名に変える。

#### ローカルでビルドしてる理由
- ローカルでビルドせずに、 `gcloud run deploy --source` でビルド～デプロイしてくれるパターンもあるけど、どうもRustの場合うまくいかなかった。
- コンソールからCloud Runのサービスを事前に作成して、「既存のコンテナイメージから１つのリビジョンをデプロイする」を選択すると、うまくデプロイできた。
- コンテナイメージをgcloudコマンドでビルド＆プッシュする方法もあるけど、これだとまたうまくいかなかった。というかビルドにすごく長い時間がかかった。基本的に10分くらいでタイムアウトするっぽいので、タイムアウトした。タイムアウト時間を延ばす設定もあるらしいけど、ちょっと長すぎていやだった。
- ローカルでイメージをビルドしてみたら、１分くらいで終わったので、ローカルでイメージ作成して、Google Container Registryにプッシュする形にした。

### イメージをプッシュ

```shell
> docker push asia.gcr.io/${PROJECT-ID}/cloudrun-1
```

#### 注意
- 上記実行時に、`gcloud` コマンドが利用できて、ログインしている状態が前提。
- [ここ](https://cloud.google.com/container-registry/docs/pushing-and-pulling)参照
- [ここ](https://cloud.google.com/container-registry/docs/advanced-authentication)に書いてありますが、下記でdockerの設定を調整しないと動作しない。

```
> gcloud auth configure-docker
```

### デプロイ

```shell
> gcloud run deploy cloudrun-1 --image asia.gcr.io/${PROJECT-ID}/cloudrun-1
```

### deproyスクリプト

```deploy.sh
docker build -t asia.gcr.io/$1/cloudrun-1 .
docker push asia.gcr.io/$1/cloudrun-1
gcloud run deploy --image asia.gcr.io/$1/cloudrun-1
```

#### 使い方

```shell
> cat .project_id | xargs sh deploy.sh
```

