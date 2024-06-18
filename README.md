# Jogo da Velha - Backend em Rust

Este é o backend para um jogo da velha multiplayer desenvolvido em Rust. A comunicação entre o servidor e os clientes é realizada via HTTP e WebSocket.

## Funcionalidades

- Suporte a partidas multiplayer;
- Comunicação em tempo real utilizando WebSocket;
- Gerenciamento de estado do jogo.

## Pré-Requisitos

Certifique-se de ter o seguinte instalado em sua máquina:

* Rust: [Instale o Rust](https://www.rust-lang.org/tools/install);
* Cargo: Gerenciador de pacotes e build system para Rust (instalado junto com Rust);

## Instalação

1. Clone este repositório

   ```bash
   git clone https://github.com/hugofsantos/jogo_da_velha_websocket
   cd jogo_da_velha_websocket
   ```

2. Instale as dependências:

   ```bash
   cargo build
   ```

## Uso

Para iniciar o servidor, execute o seguinte comando dentro do diretório raiz do projeto:

```bash
cargo run 
```