// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'institutional_identity_response.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$InstitutionalIdentityResponseCWProxy {
  InstitutionalIdentityResponse microsoftObjectId(String microsoftObjectId);

  InstitutionalIdentityResponse name(String? name);

  InstitutionalIdentityResponse email(String? email);

  InstitutionalIdentityResponse affiliation(
    InstitutionalAffiliation affiliation,
  );

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalIdentityResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalIdentityResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  InstitutionalIdentityResponse call({
    String microsoftObjectId,
    String? name,
    String? email,
    InstitutionalAffiliation affiliation,
  });
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfInstitutionalIdentityResponse.copyWith(...)` or call `instanceOfInstitutionalIdentityResponse.copyWith.fieldName(value)` for a single field.
class _$InstitutionalIdentityResponseCWProxyImpl
    implements _$InstitutionalIdentityResponseCWProxy {
  const _$InstitutionalIdentityResponseCWProxyImpl(this._value);

  final InstitutionalIdentityResponse _value;

  @override
  InstitutionalIdentityResponse microsoftObjectId(String microsoftObjectId) =>
      call(microsoftObjectId: microsoftObjectId);

  @override
  InstitutionalIdentityResponse name(String? name) => call(name: name);

  @override
  InstitutionalIdentityResponse email(String? email) => call(email: email);

  @override
  InstitutionalIdentityResponse affiliation(
    InstitutionalAffiliation affiliation,
  ) => call(affiliation: affiliation);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `InstitutionalIdentityResponse(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// InstitutionalIdentityResponse(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  InstitutionalIdentityResponse call({
    Object? microsoftObjectId = const $CopyWithPlaceholder(),
    Object? name = const $CopyWithPlaceholder(),
    Object? email = const $CopyWithPlaceholder(),
    Object? affiliation = const $CopyWithPlaceholder(),
  }) {
    return InstitutionalIdentityResponse(
      microsoftObjectId:
          microsoftObjectId == const $CopyWithPlaceholder() ||
              microsoftObjectId == null
          ? _value.microsoftObjectId
          // ignore: cast_nullable_to_non_nullable
          : microsoftObjectId as String,
      name: name == const $CopyWithPlaceholder()
          ? _value.name
          // ignore: cast_nullable_to_non_nullable
          : name as String?,
      email: email == const $CopyWithPlaceholder()
          ? _value.email
          // ignore: cast_nullable_to_non_nullable
          : email as String?,
      affiliation:
          affiliation == const $CopyWithPlaceholder() || affiliation == null
          ? _value.affiliation
          // ignore: cast_nullable_to_non_nullable
          : affiliation as InstitutionalAffiliation,
    );
  }
}

extension $InstitutionalIdentityResponseCopyWith
    on InstitutionalIdentityResponse {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfInstitutionalIdentityResponse.copyWith(...)` or `instanceOfInstitutionalIdentityResponse.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$InstitutionalIdentityResponseCWProxy get copyWith =>
      _$InstitutionalIdentityResponseCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

InstitutionalIdentityResponse _$InstitutionalIdentityResponseFromJson(
  Map<String, dynamic> json,
) => $checkedCreate('InstitutionalIdentityResponse', json, ($checkedConvert) {
  $checkKeys(
    json,
    requiredKeys: const ['microsoftObjectId', 'name', 'email', 'affiliation'],
  );
  final val = InstitutionalIdentityResponse(
    microsoftObjectId: $checkedConvert('microsoftObjectId', (v) => v as String),
    name: $checkedConvert('name', (v) => v as String?),
    email: $checkedConvert('email', (v) => v as String?),
    affiliation: $checkedConvert(
      'affiliation',
      (v) => InstitutionalAffiliation.fromJson(v as Map<String, dynamic>),
    ),
  );
  return val;
});

Map<String, dynamic> _$InstitutionalIdentityResponseToJson(
  InstitutionalIdentityResponse instance,
) => <String, dynamic>{
  'microsoftObjectId': instance.microsoftObjectId,
  'name': instance.name,
  'email': instance.email,
  'affiliation': instance.affiliation.toJson(),
};
