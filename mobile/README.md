# CApp

A new Flutter project.

## Testes Gherkin

Os cenários compartilhados estão em `../features/`. Antes da análise estática
ou dos testes, gere o arquivo usado pelo runner de integração:

```bash
dart run tool/generate_gherkin.dart
```

O runner seleciona cenários `@mobile` e registra aqueles marcados com
`@pending_mobile` como ignorados, sem executar suas steps. As tags da feature
e da regra são herdadas pelos cenários gerados. `@pending_backend` não impede
a execução no aplicativo. Uma suíte em que todos os cenários estão pendentes
continua registrando testes ignorados em vez de falhar por ausência de testes.

A CI gera os cenários antes de `flutter analyze` e verifica os filtros com
os testes de `test/`; executar os cenários de integração exige um dispositivo
ou emulador Flutter disponível.

## Getting Started

This project is a starting point for a Flutter application.

A few resources to get you started if this is your first Flutter project:

- [Learn Flutter](https://docs.flutter.dev/get-started/learn-flutter)
- [Write your first Flutter app](https://docs.flutter.dev/get-started/codelab)
- [Flutter learning resources](https://docs.flutter.dev/reference/learning-resources)

For help getting started with Flutter development, view the
[online documentation](https://docs.flutter.dev/), which offers tutorials,
samples, guidance on mobile development, and a full API reference.
