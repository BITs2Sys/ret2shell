FROM docker.io/library/python:3.12-slim
WORKDIR /app
COPY app.py .
RUN useradd -r -u 10001 koh && chown -R koh:koh /app
USER koh
EXPOSE 8080
CMD ["python3", "/app/app.py"]
