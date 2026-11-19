//! Modalidade escolhida pela comissão na criação do processo eleitoral.

/// Define quais dados podem compor votos, configuração e apuração.
///
/// Ambas as modalidades usam identidade autenticada, assinatura e participação
/// única. Somente a confidencial exige ElGamal e autoridades de decifração.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElectionModality {
    /// O registro associa a conta à opção escolhida, com acesso autorizado.
    Identified,
    /// O servidor recebe somente a escolha cifrada no dispositivo.
    Confidential,
}
