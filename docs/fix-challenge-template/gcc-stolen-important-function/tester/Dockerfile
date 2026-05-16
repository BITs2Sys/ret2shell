FROM python:3.12-alpine

COPY check.sh /check.sh
RUN chmod +x /check.sh

CMD ["/bin/sh", "-c", ". /check.sh; printf '%s=%s\\n' \"$R2S_FIX_RESULT_ENV\" \"$R2S_FIX_RESULT\""]
