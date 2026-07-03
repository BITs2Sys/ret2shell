FROM docker.io/library/gcc:13-bookworm AS build
WORKDIR /build
COPY runner.c .
RUN gcc -O2 -Wall -Wextra -o runner runner.c

FROM docker.io/library/python:3.12-slim
WORKDIR /app
COPY app.py .
COPY --from=build /build/runner /app/runner
RUN useradd -r -u 10001 koh && chown -R koh:koh /app
USER koh
EXPOSE 8080
CMD ["python3", "/app/app.py"]
