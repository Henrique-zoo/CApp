# language: pt

@audit_isolation @isolated_database
Funcionalidade: Isolamento dos bancos de teste
  Esquema do Cenário: Cada cenário possui seus próprios dados - <marcador>
    Dado que o PostgreSQL está disponível
    Quando gravo o marcador "<marcador>" no banco do cenário
    Então somente o marcador "<marcador>" está presente neste cenário

    Exemplos:
      | marcador |
      | alfa     |
      | beta     |
