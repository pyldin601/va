va-voice:
	docker build -t va-voice -f crates/app/va-voice/Dockerfile .

va-activator:
	docker build -t va-activator -f crates/app/va-activator/Dockerfile .

va-command:
	docker build -t va-command -f crates/app/va-command/Dockerfile .
