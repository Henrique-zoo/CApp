// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'institutional_auth_request.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$InstitutionalAuthRequestCWProxy {
  InstitutionalAuthRequest microsoftAccessToken(String microsoftAccessToken);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalAuthRequest(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalAuthRequest(...).copyWith(id: 12, name: "My name")
  /// ```
  InstitutionalAuthRequest call({String microsoftAccessToken});
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfInstitutionalAuthRequest.copyWith(...)` or call `instanceOfInstitutionalAuthRequest.copyWith.fieldName(value)` for a single field.
class _$InstitutionalAuthRequestCWProxyImpl
    implements _$InstitutionalAuthRequestCWProxy {
  const _$InstitutionalAuthRequestCWProxyImpl(this._value);

  final InstitutionalAuthRequest _value;

  @override
  InstitutionalAuthRequest microsoftAccessToken(String microsoftAccessToken) =>
      call(microsoftAccessToken: microsoftAccessToken);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalAuthRequest(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalAuthRequest(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  InstitutionalAuthRequest call({
    Object? microsoftAccessToken = const $CopyWithPlaceholder(),
  }) {
    return InstitutionalAuthRequest(
      microsoftAccessToken:
          microsoftAccessToken == const $CopyWithPlaceholder() ||
              microsoftAccessToken == null
          ? _value.microsoftAccessToken
          // ignore: cast_nullable_to_non_nullable
          : microsoftAccessToken as String,
    );
  }
}

extension $InstitutionalAuthRequestCopyWith on InstitutionalAuthRequest {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfInstitutionalAuthRequest.copyWith(...)` or `instanceOfInstitutionalAuthRequest.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$InstitutionalAuthRequestCWProxy get copyWith =>
      _$InstitutionalAuthRequestCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InstitutionalAuthRequest _$InstitutionalAuthRequestFromJson(
  Map<String, dynamic> json,
) => $checkedCreate('InstitutionalAuthRequest', json, ($checkedConvert) {
  $checkKeys(json, requiredKeys: const ['microsoftAccessToken']);
  final val = InstitutionalAuthRequest(
    microsoftAccessToken: $checkedConvert(
      'microsoftAccessToken',
      (v) => v as String,
    ),
  );
  return val;
});

Map<String, dynamic> _$InstitutionalAuthRequestToJson(
  InstitutionalAuthRequest instance,
) => <String, dynamic>{'microsoftAccessToken': instance.microsoftAccessToken};
