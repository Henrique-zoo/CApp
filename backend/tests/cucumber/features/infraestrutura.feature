# language: pt

@infraestrutura
Funcionalidade: Prontidão do backend
  Como responsável pela operação do sistema
  Quero verificar se o backend consegue acessar o banco de dados
  Para identificar se a aplicação está pronta para atender requisições

  @ready
  Cenário: Backend pronto quando o PostgreSQL está disponível
    Dado que o PostgreSQL está disponível
    Quando consulto a rota "/ready"
    Então o status da resposta deve ser 200
