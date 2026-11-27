//! Primitivas criptográficas do protocolo de votação confidencial.
//!
//! O CApp cifra a escolha do eleitor no dispositivo. No backend, [`elgamal`]
//! gera a chave da autoridade do servidor e valida os componentes das cifras
//! recebidas. O caso de uso de votação confere a ordem das escolhas com o
//! manifesto oficial e atualiza o acumulador a cada nova cédula aceita. Após o
//! encerramento, este módulo recupera os totais com os produtos das cifras e
//! as contribuições das autoridades.

pub mod elgamal;
