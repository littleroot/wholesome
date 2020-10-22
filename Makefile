PROJECT_ID := wholesomememe

# reference: https://www.gmosx.ninja/posts/2020/09/21/how-to-deploy-a-rust-service-to-google-cloud-run/

.PHONY: build-image
build-image:
	sudo podman build -t hello_service -f ./Dockerfile

.PHONY: run-image
run-image:
	sudo podman run -p 3000:3000 \
		-e "REDDIT_CLIENT_ID=$REDDIT_CLIENT_ID" \
		-e "REDDIT_CLIENT_SECRET=$REDDIT_CLIENT_SECRET" \
		wholesome

.PHONY: cloud-build
cloud-build:
	gcloud --project ${PROJECT_ID} builds submit --tag gcr.io/${PROJECT_ID}/wholesome

.PHONY: deploy
deploy:
	gcloud --project ${PROJECT_ID} run deploy --image gcr.io/${PROJECT_ID}/wholesome
