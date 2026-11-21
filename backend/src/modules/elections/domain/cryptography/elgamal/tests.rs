//! Regressões da criptografia eleitoral com entradas sintéticas e vetores públicos.
//!
//! O corpus exercita o mesmo caminho de agregação e apuração do backend.
//! Autenticação, assinatura do envelope e persistência ficam fora destes testes.

use std::error::Error;

use serde::Deserialize;
use serde_json::Value;

use super::*;

/// Mensagens fixadas no contrato, indexadas por hash durante o teste.
#[derive(Deserialize)]
struct Corpus {
    cases: Vec<Vector>,
    scenarios: Vec<TallyScenario>,
}

#[derive(Deserialize)]
struct Vector {
    sha256: String,
    payload: Value,
}

/// Referências ao manifesto, agregado, contribuições e resultado de cada caso.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TallyScenario {
    name: String,
    manifest_hash: String,
    aggregate_hash: String,
    share_hashes: Vec<String>,
    result_hash: String,
    voter_count: u32,
    expected_totals: Vec<u32>,
}

/// Lê números públicos do protocolo, que usa strings decimais nas fixtures.
fn decimal(value: &Value) -> U2048 {
    U2048::from_str_radix_vartime(value.as_str().expect("string decimal esperada"), 10)
        .expect("inteiro de 2048 bits esperado")
}

/// Extrai os pares na ordem publicada no vetor de teste.
fn ciphertexts(payload: &Value) -> Vec<ElGamalCiphertext> {
    payload["ciphertexts"]
        .as_array()
        .expect("vetor de cifras esperado")
        .iter()
        .map(|cipher| ElGamalCiphertext {
            a: decimal(&cipher["a"]),
            b: decimal(&cipher["b"]),
        })
        .collect()
}

/// Reconstrói cada agregado do quadro público e confere o resultado da apuração.
#[test]
fn tally_matches_shared_protocol_vectors() {
    let corpus: Corpus = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/vectors.json"
    )))
    .expect("corpus eleitoral válido");
    let by_hash: HashMap<_, _> = corpus
        .cases
        .iter()
        .map(|case| (case.sha256.as_str(), &case.payload))
        .collect();

    assert!(!corpus.scenarios.is_empty());

    for scenario in corpus.scenarios {
        let name = &scenario.name;
        let expected = ciphertexts(by_hash[scenario.aggregate_hash.as_str()]);
        let ballots: Vec<_> = corpus
            .cases
            .iter()
            .map(|case| &case.payload)
            .filter(|payload| {
                payload["kind"] == "board-entry"
                    && payload["manifestHash"] == scenario.manifest_hash.as_str()
            })
            .map(|entry| {
                EncryptedBallot::new(ciphertexts(&entry["ballot"]["payload"]))
                    .unwrap_or_else(|error| panic!("{name}: cédula do quadro inválida: {error}"))
            })
            .collect();

        assert_eq!(ballots.len(), scenario.voter_count as usize, "{name}");

        let accumulated = ballots
            .iter()
            .try_fold(
                empty_accumulator(expected.len()).unwrap(),
                accumulate_ballot,
            )
            .unwrap_or_else(|error| panic!("{name}: agregação falhou: {error}"));

        assert_eq!(accumulated.len(), expected.len(), "{name}");

        for (index, (actual, reference)) in accumulated.iter().zip(&expected).enumerate() {
            assert_eq!(
                (actual.a, actual.b),
                (reference.a, reference.b),
                "{name}: agregado divergente na escolha {index}",
            );
        }

        let contributions: Vec<Vec<_>> = scenario
            .share_hashes
            .iter()
            .map(|hash| {
                by_hash[hash.as_str()]["shares"]
                    .as_array()
                    .expect("contribuições do vetor esperadas")
                    .iter()
                    .map(|share| decimal(&share["value"]))
                    .collect()
            })
            .collect();
        let shares: Vec<_> = contributions.iter().map(Vec::as_slice).collect();
        let totals = recover_totals(&accumulated, scenario.voter_count, &shares);
        let expected_result = by_hash[scenario.result_hash.as_str()];

        match expected_result["reason"].as_str() {
            None if expected_result["status"] == "valid" => assert_eq!(
                totals.unwrap_or_else(|error| panic!("{name}: apuração falhou: {error}")),
                scenario.expected_totals,
                "{name}",
            ),
            Some("aggregate_sum_mismatch") => assert!(
                matches!(totals, Err(CryptographyError::TotalMismatch)),
                "{name}: {totals:?}",
            ),
            Some("tally_out_of_range") => assert!(
                matches!(totals, Err(CryptographyError::TotalOutOfRange)),
                "{name}: {totals:?}",
            ),
            reason => panic!("{name}: resultado sem verificação: {reason:?}"),
        }
    }
}

/// Chaves precisam ser canônicas, não triviais e pertencer ao subgrupo MODP.
#[test]
fn public_key_rejects_identity_out_of_range_and_nonmember() {
    let profile: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../packages/api-contracts/elections/profile.json"
    )))
    .unwrap();
    let group_id = profile["group"]["id"].as_str().unwrap();
    assert!(validate_public_key(group_id, &G.retrieve()).is_ok());
    assert!(matches!(
        validate_public_key("rfc3526-modp15", &G.retrieve()),
        Err(CryptographyError::UnsupportedGroup)
    ));

    let nonmember = P.as_ref().wrapping_sub(&U2048::ONE);

    for invalid in [U2048::ZERO, U2048::ONE, *P.as_ref(), nonmember] {
        assert!(matches!(
            validate_public_key(group_id, &invalid),
            Err(CryptographyError::InvalidGroupElement)
        ));
    }
}

/// O par produzido pela API pública satisfaz o intervalo e a relação público/privado.
#[test]
fn generated_authority_key_matches_its_secret() {
    let pair = generate_authority_key_pair().unwrap();

    assert!((U2048::ONE..Q).contains(pair.secret()));
    assert_eq!(*pair.public_value(), G.pow(pair.secret()).retrieve());
    assert!(validate_public_key(GROUP_ID, pair.public_value()).is_ok());
}

#[test]
fn ballot_validation_allows_identity_only_in_the_second_component() {
    let mut ciphertext = ElGamalCiphertext {
        a: G.retrieve(),
        b: U2048::ONE,
    };

    assert!(EncryptedBallot::new(vec![ciphertext]).is_ok());

    ciphertext.a = U2048::ONE;

    assert!(matches!(
        EncryptedBallot::new(vec![ciphertext]),
        Err(CryptographyError::InvalidGroupElement)
    ));
}

/// Ambos os componentes de uma cifra individual exigem pertinência ao subgrupo.
#[test]
fn ballot_rejects_nonmembers_in_both_components() {
    let nonmember = P.as_ref().wrapping_sub(&U2048::ONE);
    let g = G.retrieve();

    for (a, b) in [(nonmember, g), (g, nonmember)] {
        assert!(matches!(
            EncryptedBallot::new(vec![ElGamalCiphertext { a, b }]),
            Err(CryptographyError::InvalidGroupElement)
        ));
    }
}

/// O limite de escolhas vale tanto para o acumulador inicial como para a apuração.
#[test]
fn accumulator_and_tally_reject_choice_counts_outside_the_profile() {
    let identity = ElGamalCiphertext {
        a: U2048::ONE,
        b: U2048::ONE,
    };

    for count in [1, MAX_CHOICES + 1] {
        assert!(matches!(
            empty_accumulator(count),
            Err(CryptographyError::InvalidCiphertexts)
        ));
        assert!(matches!(
            recover_totals(&vec![identity; count], 0, &[]),
            Err(CryptographyError::InvalidCiphertexts)
        ));
    }

    assert_eq!(empty_accumulator(MAX_CHOICES).unwrap().len(), MAX_CHOICES);
}

/// Cédulas com cifras faltantes ou excedentes não produzem um acumulador novo.
#[test]
fn accumulation_rejects_wrong_ciphertext_count() {
    let ciphertext = ElGamalCiphertext {
        a: G.retrieve(),
        b: G.retrieve(),
    };

    for count in [1, 3] {
        let accumulator = empty_accumulator(2).unwrap();
        let ballot = EncryptedBallot::new(vec![ciphertext; count]).unwrap();

        assert!(matches!(
            accumulate_ballot(accumulator, &ballot),
            Err(CryptographyError::InvalidCiphertexts)
        ));
    }
}

/// Com três segredos, os votos A, A, B somam dois em A e um em B.
#[test]
fn tally_recovers_totals_in_manifest_order() {
    let secrets = [1_u8, 2, 3].map(U2048::from_u8);
    let public_key = G.pow(&U2048::from_u8(6));

    let encrypt = |selected: bool, nonce: u8| {
        let nonce = U2048::from_u8(nonce);
        let message = if selected { G } else { ONE };

        ElGamalCiphertext {
            a: G.pow(&nonce).retrieve(),
            b: message.mul(&public_key.pow(&nonce)).retrieve(),
        }
    };

    let ballots = [
        EncryptedBallot::new(vec![encrypt(true, 1), encrypt(false, 2), encrypt(false, 3)]).unwrap(),
        EncryptedBallot::new(vec![encrypt(true, 4), encrypt(false, 5), encrypt(false, 6)]).unwrap(),
        EncryptedBallot::new(vec![encrypt(false, 7), encrypt(true, 8), encrypt(false, 9)]).unwrap(),
    ];
    let accumulator = ballots
        .iter()
        .try_fold(empty_accumulator(3).unwrap(), accumulate_ballot)
        .unwrap();

    let contributions = secrets.map(|secret| {
        accumulator
            .iter()
            .map(|cipher| GroupValue::new(&cipher.a, &MOD_P).pow(&secret).retrieve())
            .collect::<Vec<_>>()
    });
    let shares = [2, 0, 1].map(|index| contributions[index].as_slice());

    assert_eq!(
        recover_totals(&accumulator, 3, &shares).unwrap(),
        vec![2, 1, 0],
    );
}

/// A apuração recusa contribuições em quantidade ou tamanho incompatível.
#[test]
fn tally_rejects_invalid_share_shape() {
    let accumulator = empty_accumulator(2).unwrap();
    let share = [U2048::ONE; 2];
    let too_few = [share.as_slice(); 2];
    let too_many = [share.as_slice(); 53];
    let truncated = [share.as_slice(), &share[..1], share.as_slice()];

    for shares in [
        too_few.as_slice(),
        too_many.as_slice(),
        truncated.as_slice(),
    ] {
        assert!(matches!(
            recover_totals(&accumulator, 0, shares),
            Err(CryptographyError::InvalidShares)
        ));
    }
}

/// Para segredos não nulos, a contribuição é identidade exatamente quando A é.
#[test]
fn tally_rejects_identity_mismatch_in_both_directions() {
    let mut accumulator = empty_accumulator(2).unwrap();
    let g = G.retrieve();
    let identity = [U2048::ONE; 2];
    let nonidentity = [g, U2048::ONE];

    assert!(matches!(
        recover_totals(&accumulator, 0, &[&nonidentity, &identity, &identity]),
        Err(CryptographyError::InvalidShares)
    ));

    accumulator[0].a = g;

    assert!(matches!(
        recover_totals(&accumulator, 0, &[identity.as_slice(); 3]),
        Err(CryptographyError::InvalidShares)
    ));
}

/// A apuração rejeita valores fora do subgrupo no agregado e nas contribuições.
#[test]
fn tally_rejects_nonmembers_in_public_inputs() {
    let mut accumulator = empty_accumulator(2).unwrap();
    let nonmember = P.as_ref().wrapping_sub(&U2048::ONE);
    let identity = [U2048::ONE; 2];

    accumulator[0].a = nonmember;
    assert!(matches!(
        recover_totals(&accumulator, 0, &[identity.as_slice(); 3]),
        Err(CryptographyError::InvalidGroupElement)
    ));

    accumulator[0].a = U2048::ONE;
    accumulator[0].b = nonmember;
    assert!(matches!(
        recover_totals(&accumulator, 0, &[identity.as_slice(); 3]),
        Err(CryptographyError::InvalidGroupElement)
    ));

    accumulator[0].a = G.retrieve();
    accumulator[0].b = U2048::ONE;
    let invalid_share = [nonmember, U2048::ONE];
    assert!(matches!(
        recover_totals(&accumulator, 0, &[invalid_share.as_slice(); 3]),
        Err(CryptographyError::InvalidGroupElement)
    ));
}

/// A busca recupera o maior expoente permitido no último bloco útil do perfil.
#[test]
fn tally_recovers_the_maximum_supported_total() {
    let mut accumulator = empty_accumulator(2).unwrap();
    let contribution = [U2048::ONE; 2];

    // Nonces somados que dão zero módulo Q deixam A = 1 no agregado.
    accumulator[0].b = G.pow(&U2048::from_u32(MAX_VOTERS)).retrieve();

    assert_eq!(
        recover_totals(&accumulator, MAX_VOTERS, &[contribution.as_slice(); 3]).unwrap(),
        [MAX_VOTERS, 0],
    );
}

/// A apuração recusa uma contagem persistida que exceda o teto do perfil.
#[test]
fn tally_rejects_voter_count_above_profile_limit() {
    let accumulator = empty_accumulator(2).unwrap();

    assert!(matches!(
        recover_totals(&accumulator, MAX_VOTERS + 1, &[]),
        Err(CryptographyError::TooManyVoters)
    ));
}

/// Uma sequência controlada cobre os limites do escalar e a máscara do sorteio.
#[test]
fn sample_non_zero_scalar_rejects_bounds_and_clears_the_high_bit() {
    let expected = Q.wrapping_sub(&U2048::ONE);
    let mut inputs = [U2048::ZERO, Q, expected].into_iter();

    let scalar = sample_non_zero_scalar(|buffer| {
        let candidate = inputs.next().expect("deveria aceitar Q - 1");
        buffer.copy_from_slice(candidate.to_be_bytes().as_slice());
        buffer[0] |= 0x80;
        Ok(())
    })
    .unwrap();

    assert_eq!(scalar, expected);
    assert!(inputs.next().is_none());
}

#[test]
fn sample_non_zero_scalar_preserves_randomness_failure() {
    let error = sample_non_zero_scalar(|_| Err(getrandom::Error::UNSUPPORTED)).unwrap_err();

    assert!(matches!(error, CryptographyError::RandomnessUnavailable(_)));
    assert_eq!(
        error
            .source()
            .and_then(|source| source.downcast_ref::<getrandom::Error>()),
        Some(&getrandom::Error::UNSUPPORTED),
    );
}
