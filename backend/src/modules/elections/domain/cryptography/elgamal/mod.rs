//! Operações ElGamal do backend para a modalidade secreta do CApp.
//!
//! Na preparação, [`generate_authority_key_pair`] gera a chave da autoridade
//! do servidor. O valor público integra o manifesto e o segredo fica sob
//! custódia dessa autoridade para produzir suas provas e contribuições.
//!
//! Durante a votação, o CApp cifra a escolha no dispositivo e envia a cédula
//! assinada. Cabe ao caso de uso conferir assinatura, elegibilidade, prazo e
//! participação única, além de comparar os IDs recebidos com a ordem do
//! manifesto oficial. [`EncryptedBallot::new`] valida os componentes ElGamal.
//!
//! Para cada nova cédula aceita, o caso de uso calcula a atualização com
//! [`accumulate_ballot`] e solicita que a persistência grave a cédula e o
//! acumulador juntos. Após o encerramento, as autoridades contribuem para o
//! agregado final. A aplicação verifica essas contribuições e usa
//! [`recover_totals`] para recuperar os votos de cada chapa e de Nulo.
//!
//! `crypto-bigint` fornece a aritmética modular de precisão fixa, e `getrandom`
//! obtém os bytes aleatórios do sistema para a chave da autoridade do servidor.

use std::collections::HashMap;

use crate::modules::elections::domain::error::CryptographyError;
use crypto_bigint::{
    Odd, U2048,
    modular::{FixedMontyForm, FixedMontyParams},
};

/// Valor em forma de Montgomery para aritmética módulo `P`.
type GroupValue = FixedMontyForm<{ U2048::LIMBS }>;

/// Identificador do grupo fixado no perfil eleitoral v1.
/// Identificador do grupo empregado nas cifras e no acumulador eleitoral.
pub(crate) const GROUP_ID: &str = "rfc3526-modp14";

/// Primo de 2048 bits do grupo MODP 14.
const P: Odd<U2048> = Odd::<U2048>::from_be_hex(concat!(
    "ffffffffffffffffc90fdaa22168c234c4c6628b80dc1cd129024e088a67cc74",
    "020bbea63b139b22514a08798e3404ddef9519b3cd3a431b302b0a6df25f1437",
    "4fe1356d6d51c245e485b576625e7ec6f44c42e9a637ed6b0bff5cb6f406b7ed",
    "ee386bfb5a899fa5ae9f24117c4b1fe649286651ece45b3dc2007cb8a163bf05",
    "98da48361c55d39a69163fa8fd24cf5f83655d23dca3ad961c62f356208552bb",
    "9ed529077096966d670c354e4abc9804f1746c08ca18217c32905e462e36ce3b",
    "e39e772c180e86039b2783a2ec07a28fb5c55df06f4c52c9de2bcbf695581718",
    "3995497cea956ae515d2261898fa051015728e5a8aacaa68ffffffffffffffff",
));

/// Parâmetro público compartilhado com o verificador de provas do protocolo.
pub(crate) fn group_modulus() -> U2048 {
    *P.as_ref()
}

/// Parâmetros de Montgomery calculados na compilação para o primo público.
const MOD_P: FixedMontyParams<{ U2048::LIMBS }> = FixedMontyParams::new_vartime(P);

/// Ordem do subgrupo, igual a `(P - 1) / 2` para o primo ímpar `P`.
const Q: U2048 = P.as_ref().shr(1);

/// Gerador do subgrupo de ordem `Q`.
const G: GroupValue = GroupValue::new(&U2048::from_u8(2), &MOD_P);

/// Identidade multiplicativa do subgrupo, em forma de Montgomery.
const ONE: GroupValue = GroupValue::one(&MOD_P);

/// Teto de cédulas aceitas por eleição no perfil v1.
///
/// O caso de uso de votação aplica este limite antes de aceitar outra cédula.
/// [`recover_totals`] também o confere antes de calcular os totais.
pub const MAX_VOTERS: u32 = 100_000;

/// Teto de posições do perfil v1, contando até cinquenta chapas e Nulo.
const MAX_CHOICES: usize = 51;

/// Par ElGamal da autoridade do servidor para uma eleição.
///
/// O manifesto publica o valor público. A autoridade conserva o segredo para
/// provar que conhece a chave e gerar sua contribuição para a apuração.
pub struct ElGamalAuthorityKeyPair {
    secret: U2048,
    public_value: U2048,
}

impl ElGamalAuthorityKeyPair {
    /// Escalar privado `x` em `[1, Q)`, sob custódia da autoridade.
    pub fn secret(&self) -> &U2048 {
        &self.secret
    }

    /// Elemento público `g^x mod P`, a ser incluído no manifesto.
    pub fn public_value(&self) -> &U2048 {
        &self.public_value
    }
}

/// Gera o par ElGamal da autoridade do servidor durante a preparação da eleição.
///
/// A cada chamada, obtém bytes aleatórios do sistema operacional e sorteia
/// uniformemente um segredo diferente de zero. Antes de publicar o manifesto,
/// o caso de uso chama esta função novamente se o produto das chaves públicas
/// der `1`, e refaz a prova da autoridade do servidor. O par definitivo fica
/// sob custódia do servidor para a apuração.
///
/// # Retorno
///
/// Par com o segredo `x` em `[1, Q)` e o valor público `g^x mod P`.
///
/// # Erros
///
/// Retorna [`CryptographyError::RandomnessUnavailable`] se o sistema operacional
/// falhar ao fornecer os bytes aleatórios para o sorteio.
pub fn generate_authority_key_pair() -> Result<ElGamalAuthorityKeyPair, CryptographyError> {
    let secret = sample_non_zero_scalar(getrandom::fill)?;

    let public_value = G.pow(&secret).retrieve();

    Ok(ElGamalAuthorityKeyPair {
        secret,
        public_value,
    })
}

/// Sorteia por rejeição um escalar para gerar a chave da autoridade.
///
/// # Parâmetros
///
/// - `fill_random` preenche todo o buffer de 256 bytes a cada tentativa.
///   A geração da chave usa a fonte segura do sistema. Os testes podem
///   fornecer bytes conhecidos para exercitar as rejeições.
///
/// # Retorno
///
/// Um inteiro em `[1, Q)`, uniforme quando a fonte fornece bytes uniformes.
/// A função limpa o bit mais alto e descarta zero e valores a partir de `Q`.
///
/// # Erros
///
/// Retorna [`CryptographyError::RandomnessUnavailable`] se a fonte falhar.
fn sample_non_zero_scalar(
    mut fill_random: impl FnMut(&mut [u8]) -> Result<(), getrandom::Error>,
) -> Result<U2048, CryptographyError> {
    let mut bytes = [0_u8; 256];

    loop {
        fill_random(&mut bytes)?;

        bytes[0] &= 0x7F;

        let candidate = U2048::from_be_slice(&bytes);

        if candidate != U2048::ZERO && candidate < Q {
            return Ok(candidate);
        }
    }
}

/// Par de componentes ElGamal de uma posição da cédula ou do agregado.
///
/// Em uma cédula, `m` é o bit daquela escolha, `r` é o nonce da posição e
/// `h` é a chave pública conjunta. No agregado, os campos contêm os produtos
/// dos componentes da mesma escolha nas cédulas aceitas.
#[derive(Clone, Copy)]
pub struct ElGamalCiphertext {
    /// Primeiro componente. Na cédula individual, vale `g^r mod P`.
    pub a: U2048,

    /// Segundo componente. Na cédula individual, vale `g^m * h^r mod P`.
    pub b: U2048,
}

/// Sequência de cifras de uma cédula com componentes validados.
///
/// O caso de uso confere a ordem das escolhas e decide se aceita a cédula.
/// Só as cédulas aceitas até o encerramento entram na agregação.
pub struct EncryptedBallot {
    ciphertexts: Vec<ElGamalCiphertext>,
}

impl EncryptedBallot {
    /// Valida os componentes ElGamal de uma cédula individual.
    ///
    /// # Parâmetros
    ///
    /// - `ciphertexts` contém os pares recebidos na ordem conferida pelo
    ///   chamador. Ambos os componentes devem pertencer ao subgrupo. O
    ///   primeiro componente de cada par também deve ser diferente de `1`.
    ///
    /// # Retorno
    ///
    /// Cédula com os mesmos pares na ordem fornecida, pronta para agregação
    /// depois de aceita. A validação não comprova que as cifras contêm bits.
    ///
    /// # Erros
    ///
    /// Retorna [`CryptographyError::InvalidGroupElement`] se algum componente
    /// violar a faixa, o subgrupo ou a regra de identidade.
    pub fn new(ciphertexts: Vec<ElGamalCiphertext>) -> Result<Self, CryptographyError> {
        for cipher in &ciphertexts {
            validate_element(&cipher.a, false)?;
            validate_element(&cipher.b, true)?;
        }

        Ok(Self { ciphertexts })
    }

    /// Cifras na ordem em que foram fornecidas para validação.
    pub fn ciphertexts(&self) -> &[ElGamalCiphertext] {
        &self.ciphertexts
    }
}

/// Confere o grupo e a chave pública ElGamal no perfil eleitoral v1.
///
/// O chamador usa a função para validar os valores públicos das autoridades
/// e a chave conjunta ao preparar ou ler um manifesto.
///
/// # Parâmetros
///
/// - `group_id` é o identificador do grupo escolhido para o manifesto.
/// - `public_key` é o inteiro público antes de qualquer redução módulo `P`.
///
/// # Retorno
///
/// `Ok(())` se a chave pertence ao subgrupo e é diferente de `1`.
///
/// # Erros
///
/// - [`CryptographyError::UnsupportedGroup`] se `group_id` divergir do perfil v1.
/// - [`CryptographyError::InvalidGroupElement`] se a chave for a identidade ou
///   estiver fora da faixa ou do subgrupo.
pub fn validate_public_key(group_id: &str, public_key: &U2048) -> Result<(), CryptographyError> {
    if group_id != GROUP_ID {
        return Err(CryptographyError::UnsupportedGroup);
    }

    validate_element(public_key, false).map(|_| ())
}

/// Confere a pertinência de um valor público ao subgrupo do perfil v1.
///
/// # Parâmetros
///
/// - `value` é o inteiro recebido, antes de qualquer redução módulo `P`.
/// - `allow_identity` indica se o campo aceita `1`. A chave conjunta e o
///   primeiro componente de uma cifra individual exigem um valor diferente
///   de `1`. O segundo componente e os elementos agregados podem aceitá-lo.
///
/// # Retorno
///
/// O elemento na forma de Montgomery, pronto para operações módulo `P`.
///
/// # Erros
///
/// Retorna [`CryptographyError::InvalidGroupElement`] se `value` estiver fora de
/// `[1, P)`, se `value^Q mod P` divergir de `1` ou se a identidade estiver vedada.
fn validate_element(value: &U2048, allow_identity: bool) -> Result<GroupValue, CryptographyError> {
    if *value == U2048::ZERO || value >= P.as_ref() || (!allow_identity && *value == U2048::ONE) {
        return Err(CryptographyError::InvalidGroupElement);
    }

    let element = GroupValue::new(value, &MOD_P);

    if element.pow_vartime(&Q) != ONE {
        return Err(CryptographyError::InvalidGroupElement);
    }

    Ok(element)
}

/// Inicializa o acumulador de uma eleição que ainda não tem cédulas aceitas.
///
/// # Parâmetros
///
/// - `choice_count` é a quantidade de escolhas do manifesto, incluindo Nulo.
///
/// # Retorno
///
/// Um par `(1, 1)` por posição do manifesto. Este também é o agregado final
/// quando nenhuma cédula é aceita antes do encerramento.
///
/// # Erros
///
/// Retorna [`CryptographyError::InvalidCiphertexts`] se `choice_count` estiver
/// fora de `2..=51`.
pub fn empty_accumulator(choice_count: usize) -> Result<Vec<ElGamalCiphertext>, CryptographyError> {
    if !(2..=MAX_CHOICES).contains(&choice_count) {
        return Err(CryptographyError::InvalidCiphertexts);
    }

    Ok(vec![
        ElGamalCiphertext {
            a: U2048::ONE,
            b: U2048::ONE,
        };
        choice_count
    ])
}

/// Incorpora as cifras de uma cédula ao acumulador da eleição.
///
/// Para cada posição, multiplica separadamente os primeiros e os segundos
/// componentes, módulo `P`. O caso de uso chama a função para uma cédula
/// nova e grava a atualização junto com o aceite e a contagem de cédulas.
///
/// # Parâmetros
///
/// - `accumulator` é o vetor de produtos obtidos desde [`empty_accumulator`],
///   na ordem do manifesto. A função recebe a propriedade do vetor. O caso
///   de uso carrega esses valores da mesma revisão que usará para confirmar
///   a gravação.
/// - `ballot` contém as cifras validadas de uma cédula da mesma eleição e já
///   conferidas na ordem do manifesto. O chamador garante que a cédula ainda
///   não faz parte do acumulador.
///
/// # Retorno
///
/// O vetor com os produtos atualizados, reutilizando a alocação recebida.
/// A contagem é mantida pelo chamador.
///
/// # Erros
///
/// Retorna [`CryptographyError::InvalidCiphertexts`] se o acumulador estiver
/// fora de `2..=51` posições ou se o tamanho da cédula diferir. Nesse caso,
/// o vetor de entrada é descartado e nenhum acumulador é retornado.
///
/// # Exemplos
///
/// Ao incorporar a primeira cédula de uma eleição fictícia, cada par passa
/// de `(1, 1)` para o par da mesma posição na cédula.
///
/// ```
/// use backend::modules::elections::domain::cryptography::elgamal::{
///     ElGamalCiphertext, EncryptedBallot, accumulate_ballot, empty_accumulator,
/// };
/// use crypto_bigint::U2048;
///
/// let accumulator = empty_accumulator(2)?;
/// let ciphertexts = [
///     ElGamalCiphertext { a: U2048::from_u8(2), b: U2048::from_u8(8) },
///     ElGamalCiphertext { a: U2048::from_u8(4), b: U2048::from_u8(16) },
/// ];
/// let ballot = EncryptedBallot::new(ciphertexts.to_vec())?;
/// let accumulator = accumulate_ballot(accumulator, &ballot)?;
/// for (product, cipher) in accumulator.iter().zip(ciphertexts) {
///     assert_eq!((product.a, product.b), (cipher.a, cipher.b));
/// }
/// # Ok::<(), backend::modules::elections::domain::error::CryptographyError>(())
/// ```
pub fn accumulate_ballot(
    mut accumulator: Vec<ElGamalCiphertext>,
    ballot: &EncryptedBallot,
) -> Result<Vec<ElGamalCiphertext>, CryptographyError> {
    let choice_count = accumulator.len();

    if !(2..=MAX_CHOICES).contains(&choice_count) || choice_count != ballot.ciphertexts.len() {
        return Err(CryptographyError::InvalidCiphertexts);
    }

    accumulator
        .iter_mut()
        .zip(&ballot.ciphertexts)
        .for_each(|(product, cipher)| {
            let product_a = GroupValue::new(&product.a, &MOD_P);
            let product_b = GroupValue::new(&product.b, &MOD_P);
            let a = GroupValue::new(&cipher.a, &MOD_P);
            let b = GroupValue::new(&cipher.b, &MOD_P);

            product.a = product_a.mul(&a).retrieve();
            product.b = product_b.mul(&b).retrieve();
        });

    Ok(accumulator)
}

/// Recupera os totais de cada escolha a partir do agregado e das contribuições.
///
/// Para cada escolha, divide o segundo componente do agregado pelo produto
/// das contribuições módulo `P`. O resultado da decifração é `g^T mod P`, com
/// o total de votos `T` ainda no expoente. A função usa uma tabela compartilhada
/// para recuperar esse inteiro para cada escolha.
///
/// # Parâmetros
///
/// - `aggregate` contém os produtos finais das cifras, na ordem do manifesto.
///   O chamador obtém esses produtos e `voter_count` do mesmo encerramento.
/// - `voter_count` é o número de cédulas aceitas e incluídas no agregado.
/// - `shares` contém uma contribuição por autoridade do manifesto, em qualquer
///   ordem. Cada contribuição contém um valor `D_i = A^x_i mod P` por escolha,
///   incluindo Nulo, na mesma ordem de `aggregate`. `A` é o primeiro componente
///   do agregado naquela posição e `x_i` é o segredo da autoridade.
///
/// Antes de chamar a função, a aplicação garante que há uma contribuição por
/// autoridade esperada, sem duplicatas. Também confere a ordem das escolhas,
/// a assinatura e a prova vinculada ao agregado e ao encerramento.
///
/// # Retorno
///
/// Um total por posição do manifesto, incluindo escolhas sem votos e Nulo.
/// O chamador associa os totais aos IDs das escolhas pela ordem do manifesto.
/// Em caso de sucesso, a soma dos totais é igual a `voter_count`. Essa igualdade
/// confere a contagem agregada. Cédulas inválidas que se compensam ainda podem
/// satisfazê-la.
///
/// # Erros
///
/// - [`CryptographyError::InvalidCiphertexts`] se o agregado tiver menos de
///   duas ou mais de cinquenta e uma posições.
/// - [`CryptographyError::TooManyVoters`] se a contagem exceder [`MAX_VOTERS`].
/// - [`CryptographyError::InvalidShares`] se a quantidade de contribuições
///   estiver fora de `3..=52`, se uma contribuição tiver quantidade de valores
///   diferente do agregado ou se `D_i = 1` e `A != 1`, ou vice-versa.
/// - [`CryptographyError::InvalidGroupElement`] se um valor estiver fora da
///   faixa ou do subgrupo, ou se um produto necessário não for invertível.
/// - [`CryptographyError::TotalOutOfRange`] se algum elemento recuperado não
///   corresponder a um total em `[0, voter_count]`.
/// - [`CryptographyError::TotalMismatch`] se a soma divergir de `voter_count`.
///
/// # Como a tabela recupera a contagem
///
/// Uma busca direta compararia o valor decifrado com `g^0`, `g^1` e assim por
/// diante até `g^voter_count`, sempre módulo `P`. O *baby-step giant-step*
/// divide os **totais possíveis** em blocos. Um bloco é um intervalo de
/// contagens consecutivas: com `width = 4`, o bloco 0 vai de 0 a 3, o bloco 1
/// vai de 4 a 7 e o bloco 2 vai de 8 a 11. Em geral, o bloco `k` começa em
/// `k * width`. Um total desse bloco tem a forma `T = k * width + j`, em que
/// `j` é a posição dentro do bloco, de 0 até `width - 1`.
///
/// A tabela guarda **potência → posição no bloco**. Para `width = 4`, suas
/// entradas são `g^0 → 0`, `g^1 → 1`, `g^2 → 2` e `g^3 → 3`, módulo `P`.
/// Para testar o bloco `k`, multiplicamos o valor decifrado por
/// `g^(-k * width)`. Isso tira do expoente o início do bloco: se `T` estiver
/// nele, o resultado será `g^j`. A consulta à tabela devolve `j` e o total é
/// `k * width + j`. Se não houver entrada, passamos ao bloco seguinte
/// multiplicando mais uma vez por `g^(-width)`.
///
/// Por exemplo, com 11 cédulas aceitas e valor decifrado `g^11 mod P`:
///
/// | Bloco | Totais possíveis no bloco | Potência consultada | Resposta da tabela |
/// | --- | --- | --- | --- |
/// | 0 | 0 a 3 | `g^11` | Ausente |
/// | 1 | 4 a 7 | `g^7` | Ausente |
/// | 2 | 8 a 11 | `g^3` | Posição `3` |
///
/// A terceira consulta encontra o total `2 * 4 + 3 = 11`. A mesma tabela
/// atende todos os blocos e todas as escolhas da eleição. Como cada consulta
/// testa até `width` totais, `width` blocos cobrem `width²` possibilidades.
/// Escolher a menor largura com `width² >= voter_count + 1` equilibra o número
/// de entradas com o de consultas, incluindo o total zero. Para um máximo de
/// 100 mil cédulas, são no máximo 317 entradas.
///
/// Referência: [*Handbook of Applied Cryptography*, §3.6.2, algoritmo 3.56](https://cacr.uwaterloo.ca/hac/about/chap3.pdf).
pub fn recover_totals(
    aggregate: &[ElGamalCiphertext],
    voter_count: u32,
    shares: &[&[U2048]],
) -> Result<Vec<u32>, CryptographyError> {
    if voter_count > MAX_VOTERS {
        return Err(CryptographyError::TooManyVoters);
    }

    if !(2..=MAX_CHOICES).contains(&aggregate.len()) {
        return Err(CryptographyError::InvalidCiphertexts);
    }

    if !(3..=52).contains(&shares.len()) {
        return Err(CryptographyError::InvalidShares);
    }

    if shares.iter().any(|share| share.len() != aggregate.len()) {
        return Err(CryptographyError::InvalidShares);
    }

    // Remove o fator de cifragem para obter g^T por escolha, incluindo Nulo.
    let encoded_totals = aggregate
        .iter()
        .enumerate()
        .map(|(index, cipher)| {
            validate_element(&cipher.a, true)?;

            // Calcula o produtório dos Ds fornecidos pelas autoridades para esse candidato.
            let product = shares.iter().try_fold(ONE, |product, &share| {
                let value = &share[index];

                if (*value == U2048::ONE) != (cipher.a == U2048::ONE) {
                    return Err(CryptographyError::InvalidShares);
                }

                Ok(product.mul(&validate_element(value, true)?))
            })?;

            let b = validate_element(&cipher.b, true)?;

            Ok(b.mul(&invert_public(&product)?))
        })
        .collect::<Result<Vec<_>, CryptographyError>>()?;

    // A menor largura que cobre 0..=voter_count em até width blocos.
    let width = voter_count.isqrt() + 1;
    let mut baby_steps = HashMap::with_capacity(width as usize);
    let mut power = ONE;

    // Guardamos cada potência em Montgomery com sua posição no bloco.
    for offset in 0..width {
        baby_steps.insert(*power.as_montgomery(), offset);
        power = power.mul(&G);
    }

    // Ao fim do laço, power = g^width. Seu inverso subtrai width do expoente
    // a cada passo da busca.
    let backward = invert_public(&power)?;
    let totals = encoded_totals
        .into_iter()
        .map(|message| bounded_log(message, voter_count, width, &baby_steps, &backward))
        .collect::<Result<Vec<_>, _>>()?;

    // Esse caso deve cancelar a eleição.
    if totals.iter().sum::<u32>() != voter_count {
        return Err(CryptographyError::TotalMismatch);
    }

    Ok(totals)
}

/// Retorna o inverso multiplicativo de `value` módulo `P`.
///
/// A apuração usa a inversão para remover o produto das contribuições de cada
/// escolha e para calcular o fator que desloca a busca entre blocos. Como os
/// valores são públicos, a função usa a variante de tempo variável.
///
/// Retorna [`CryptographyError::InvalidGroupElement`] se não houver inverso.
fn invert_public(value: &GroupValue) -> Result<GroupValue, CryptographyError> {
    value
        .invert_vartime()
        .into_option()
        .ok_or(CryptographyError::InvalidGroupElement)
}

/// Recupera o expoente de um elemento público em `[0, voter_count]`.
///
/// Usa a tabela construída por [`recover_totals`]. No início do bloco `k`,
/// `candidate` contém `g^(T - k * width) mod P`. O laço faz no máximo
/// `voter_count / width + 1` consultas, com divisão inteira. Como `g` tem
/// ordem `Q` e `voter_count < Q`, totais distintos no intervalo permitido
/// correspondem a potências distintas. O último bloco pode conter posições
/// acima de `voter_count`: por exemplo, se o limite é 10 e `width = 4`,
/// o bloco que começa em 8 também contém 11, que precisa ser rejeitado.
///
/// # Parâmetros
///
/// - `candidate` começa com o elemento `g^T mod P` de uma escolha.
/// - `voter_count` é o número de cédulas incluídas e o maior total permitido.
/// - `width` é a largura positiva de cada bloco de expoentes.
/// - `baby_steps` associa, para cada `j` em `[0, width)`, a representação
///   de Montgomery de `g^j mod P` ao expoente `j`.
/// - `backward` é o inverso de `g^width mod P`.
///
/// # Retorno
///
/// O expoente `T` quando ele pertence a `[0, voter_count]`. A função retorna
/// [`CryptographyError::TotalOutOfRange`] quando a busca não encontra um
/// expoente nesse intervalo.
fn bounded_log(
    mut candidate: GroupValue,
    voter_count: u32,
    width: u32,
    baby_steps: &HashMap<U2048, u32>,
    backward: &GroupValue,
) -> Result<u32, CryptographyError> {
    for block in 0..=voter_count / width {
        if let Some(&offset) = baby_steps.get(candidate.as_montgomery()) {
            let count = block * width + offset;

            if count <= voter_count {
                return Ok(count);
            }
        }

        candidate = candidate.mul(backward);
    }

    Err(CryptographyError::TotalOutOfRange)
}

#[cfg(test)]
mod tests;
