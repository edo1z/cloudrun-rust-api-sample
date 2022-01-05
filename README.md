## Cloud RunにRustプログラムをデプロイする

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
> docker build -t gcr.io/${PROJECT-ID}/${SERVICE_NAME} .
```

#### 注意
- `gcr.io` は、Container Registryで設定しているドメインによって変える。
- `${PROJECT-ID}`は、自分のプロジェクトIDに変える。
- `${SERVICE_NAME}`は、自分のサービス名に変える。

#### ローカルでビルドしてる理由
- ローカルでビルドせずに、 `gcloud run deploy --source` でビルド～デプロイしてくれるパターンもあるけど、どうもRustの場合うまくいかなかった。
- コンソールからCloud Runのサービスを事前に作成して、「既存のコンテナイメージから１つのリビジョンをデプロイする」を選択すると、うまくデプロイできた。
- コンテナイメージをgcloudコマンドでビルド＆プッシュする方法もあるけど、これだとまたうまくいかなかった。というかビルドにすごく長い時間がかかった。基本的に10分くらいでタイムアウトするっぽいので、タイムアウトした。タイムアウト時間を延ばす設定もあるらしいけど、ちょっと長すぎていやだった。
- ローカルでイメージをビルドしてみたら、１分くらいで終わったので、ローカルでイメージ作成して、Google Container Registryにプッシュする形にした。

### イメージをプッシュ

```shell
> docker push gcr.io/${PROJECT-ID}/${SERVICE_NAME}
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
> gcloud run deploy ${SERVICE_NAME} --image gcr.io/${PROJECT-ID}/${SERVICE_NAME}
```

### 参考: deproyスクリプト
#### .cloudrun_env
- 下記のようなPROJECT_ID等を設定するshellスクリプトを作成して、読み込みます。
- ESP_***という変数は、下記のEndpoint設定時に利用します。

```.cloudrun_env
export SERVICE_NAME=*****
export PROJECT_ID=*****

export ESP_SERVICE_NAME=*****
export ESP_PROJECT_ID=*****
export ESP_HOSTNAME=*****
```

#### .cloudrun_envの読み込み

```shell
> source .cloudrun_env
```

#### scripts/deploy.sh

```deploy.sh
docker build -t gcr.io/${PROJECT_ID}/${SERVICE_NAME} .
docker push gcr.io/${PROJECT_ID}/${SERVICE_NAME}
gcloud run deploy ${SERVICE_NAME} \
  --image gcr.io/${PROJECT_ID}/${SERVICE_NAME} \
  --no-allow-unauthenticated
```

- 上記の`no-allow-unauthenticated`は、未認証のアクセスを許可しない設定。
- 許可する場合は、`allow-unauthenticated`にする。

#### ビルド～デプロイ

```shell
> sh scripts/deploy.sh
```

## Cloud Endpointsの設定・デプロイ
- Cloud Endpointsは、APIの管理とアクセス制御等をしてくれます。
  - Swagger準拠のyamlをアップすることで、自動的にAPIの受け口と、ドキュメント作成をしてくれる。
  - 認証・CORS関連も設定できる。
  - アクセス数制限とかもできる。
- 基本的には、[ここ](https://cloud.google.com/endpoints/docs/openapi/get-started-cloud-run)を見ながら設定します。

### ESPv2用のCloud Runのサービスを作る

- Googleが用意してくれている、ダミーのイメージで、とりあえずESPv2用のサービスを作る。（下記のESP_SERVICE_NAMEと、ESP_HOSTNAMEを以降の設定で利用するため）

```shell
> gcloud run deploy ${ESP_SERVICE_NAME} --image="gcr.io/cloudrun/hello"
Service URL: https://[ESP_HOSTNAME]
```

### Endpoints構成をデプロイ

- Swaggerの定義ファイル（Endpoint構成）を作成したら下記でデプロイする。

```shell
> gcloud endpoints services deploy openapi-run.yaml --project ${ESP_PROJECT_ID}
Service Configuration [CONFIG_ID] uploaded for service [ESP_HOSTNAME]
```

- 上記のESP_PROJECT_IDは、PROJECT_IDと同じ。（今回は同じProjectにESPv2も入れているため）
- 上記結果として表示される、`CONFIG_ID` を下記で使用します。

### ESPv2イメージをビルド・プッシュ

```shell
> ./gcloud_build_image -s ${ESP_HOSTNAME} -c ${CONFIG_ID} -p ${ESP_PROJECT_ID}
```
- 上記結果として、 下記形式のESPv2イメージのURLが表示されます。これを下記で使用します。
  - `gcr.io/[ESP_PROJECT_ID]/endpoints-runtime-serverless:[ESP_VERSION]-[ESP_HOSTNAME]-[CONFIG_ID]`

### ESPv2コンテナをデプロイ

```shell
> gcloud run deploy ${ESP_SERVICE_NAME} \
  --image="${ESPv2 image url}" --project=${ESP_PROJECT_ID} --set-env-vars=ESPv2_ARGS=--cors_preset=basic
```

- 上記はCORSの基本設定もしています。詳細は[ここ](https://cloud.google.com/endpoints/docs/openapi/specify-esp-v2-startup-options#cors)に書いてあります。

### 参考: ESPv2のdeployスクリプト
#### scripts/esp_deploy.sh

```shell
gcloud endpoints services deploy openapi-run.yaml --project ${ESP_PROJECT_ID} 
gcloud endpoints configs list --service=${ESP_HOSTNAME} | head -n 2 | tail -n 1 | grep -oP '\S+' -m1 | head -n 1 > tmp_config_id 
CONFIG_ID=`cat tmp_config_id`
./scripts/gcloud_build_image -s ${ESP_HOSTNAME} -c ${CONFIG_ID} -p ${ESP_PROJECT_ID}
rm tmp_config_id
gcloud container images list-tags gcr.io/${ESP_PROJECT_ID}/endpoints-runtime-serverless | head -n 2 | tail -n 1 | grep -oP '\S+' -m1 | head -n 2 | tail -n 1 > tmp_image_tag
TAG=`cat tmp_image_tag`
IMAGE="gcr.io/${ESP_PROJECT_ID}/endpoints-runtime-serverless:${TAG}"
gcloud run deploy ${ESP_SERVICE_NAME} --image=${IMAGE} --project=${ESP_PROJECT_ID} --set-env-vars=ESPv2_ARGS=--cors_preset=basic
rm tmp_image_tag
```

### Cloud Runに直接アクセスできないようにする
- Cloud RunはEndpoint経由でのみアクセスできるようにします。

```shell
> gcloud run deploy ${SERVICE_NAME} --image gcr.io/${PROJECT-ID}/${SERVICE_NAME} --no-allow-unauthenticated
```