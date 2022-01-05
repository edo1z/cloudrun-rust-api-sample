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
