//! Limites do perfil eleitoral v1 compartilhados pelo domínio e pelo ElGamal.

/// Chapa única acompanhada da opção Nulo.
pub const MIN_OPTIONS: usize = 2;

/// Até cinquenta chapas acompanhadas da opção Nulo.
pub const MAX_OPTIONS: usize = 51;

/// Servidor, comissão e uma chapa.
pub const MIN_AUTHORITIES: usize = MIN_OPTIONS + 1;

/// Servidor, comissão e uma autoridade por chapa.
pub const MAX_AUTHORITIES: usize = MAX_OPTIONS + 1;

/// Limite de bytes da representação canônica de um artefato assinado.
pub const MAX_SIGNED_MESSAGE_BYTES: usize = 262_144;

/// Maior quantidade de votos aceitos por eleição.
pub const MAX_VOTER_COUNT: u32 = 100_000;
