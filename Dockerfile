FROM alpine:3.24 AS builder
WORKDIR /builder
COPY . .
RUN ./scripts/binary.sh $TARGETARCH

FROM scratch
COPY --from=builder --chmod=755 /builder/cli ./cli