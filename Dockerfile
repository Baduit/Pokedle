FROM ubuntu:22.04 
WORKDIR /pokedle
COPY . /pokedle/

ENV DEBIAN_FRONTEND noninteractive
RUN apt update

RUN apt install curl build-essential python3 python3-pip  -y
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo build --release
RUN cp target/release/libpokedle.so ./pokedle.so

RUN pip install "fastapi[all]" fastapi_utils "uvicorn[standard]" pyserde


CMD ["uvicorn", "pokedle_back:app", "--host", "0.0.0.0", "--port", "3412"]

EXPOSE 3412