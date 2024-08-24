FROM alpine:latest AS builder
ARG TARGETARCH
WORKDIR /builder
COPY . .
RUN ./scripts/binary.sh $TARGETARCH

FROM scratch
COPY --from=builder --chmod=755 /builder/cli ./cli