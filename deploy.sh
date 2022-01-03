docker build -t asia.gcr.io/$1/cloudrun-1 .
docker push asia.gcr.io/$1/cloudrun-1
gcloud run deploy cloudrun-1 --image asia.gcr.io/$1/cloudrun-1