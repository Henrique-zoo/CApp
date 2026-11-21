//! Falhas das operações de domínio de eleições.
//!
//! Os erros identificam a etapa que falhou e preservam a causa técnica quando
//! ela existe. A aplicação decide como tratar a falha no fluxo eleitoral, e a
//! camada API decide como comunicá-la ao cliente.

/// Falhas das operações criptográficas do protocolo eleitoral.
#[derive(Debug, thiserror::Error)]
pub enum CryptographyError {
    /// A fonte de aleatoriedade do sistema falhou ao preencher o buffer.
    ///
    /// A causa é preservada para diagnóstico. A operação deve ser interrompida
    /// porque o buffer pode ter sido preenchido apenas em parte.
    #[error("Não foi possível obter bytes aleatórios para a operação criptográfica")]
    RandomnessUnavailable(#[from] getrandom::Error),

    /// O número de escolhas viola o perfil ou difere da quantidade de cifras.
    #[error("Quantidade de cifras incompatível com as escolhas esperadas")]
    InvalidCiphertexts,

    /// O identificador do grupo não é o fixado pelo perfil eleitoral.
    #[error("Grupo ElGamal não suportado pelo perfil eleitoral")]
    UnsupportedGroup,

    /// Um valor viola as regras do subgrupo ou de identidade.
    #[error("Elemento ElGamal inválido")]
    InvalidGroupElement,

    /// A contagem de cédulas aceitas excede o limite do perfil eleitoral.
    #[error("Limite de cédulas excedido")]
    TooManyVoters,

    /// O conjunto de contribuições está incompleto ou é incompatível.
    #[error("Contribuições incompatíveis com o agregado")]
    InvalidShares,

    /// Um elemento recuperado não representa nenhum total em `[0, n]`.
    #[error("Total fora do intervalo de cédulas aceitas")]
    TotalOutOfRange,

    /// A soma dos totais diverge do número de cédulas aceitas.
    #[error("Soma dos totais divergente da contagem de cédulas")]
    TotalMismatch,
}
