FROM ubuntu
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    zsh \
    jq
RUN which zsh | tee -a /etc/shells
RUN chsh -s $(which zsh)

# Not sure how to build from source
# RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
# COPY . .
# RUN . $HOME/.cargo/env;cargo install --path .

# But these two lines grab the 1.0.1 (latest) release
RUN curl -SsL -o /usr/bin/rpg-cli https://github.com/facundoolano/rpg-cli/releases/download/1.0.1/rpg-cli-1.0.1-linux
RUN chmod a+x /usr/bin/rpg-cli
WORKDIR /root

COPY .zshrc.rpg .zshrc
ENTRYPOINT ["zsh", "-c", ". ~/.zshrc;playself"]
