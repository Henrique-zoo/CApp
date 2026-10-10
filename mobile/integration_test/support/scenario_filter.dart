/// Identifica cenários cuja automação mobile ainda não está disponível.
///
/// O compilador Gherkin inclui as tags herdadas da feature e da regra nos
/// cenários gerados. A pendência do backend não impede a execução no mobile.
bool isPendingMobileScenario(Iterable<String> tags) {
  return tags.contains('@pending_mobile');
}
