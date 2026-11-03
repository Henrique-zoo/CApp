// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'api_error.dart';

// **************************************************************************
// CopyWithGenerator
// **************************************************************************

abstract class _$ApiErrorCWProxy {
  ApiError error(ApiErrorDetails error);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `ApiError(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// ApiError(...).copyWith(id: 12, name: "My name")
  /// ```
  ApiError call({ApiErrorDetails error});
}

/// Callable proxy for `copyWith` functionality.
/// Use as `instanceOfApiError.copyWith(...)` or call `instanceOfApiError.copyWith.fieldName(value)` for a single field.
class _$ApiErrorCWProxyImpl implements _$ApiErrorCWProxy {
  const _$ApiErrorCWProxyImpl(this._value);

  final ApiError _value;

  @override
  ApiError error(ApiErrorDetails error) => call(error: error);

  /// Creates a new instance with the provided field values.
  /// Omitted fields keep their values; explicit `null` clears nullable fields.
  /// The public API rejects `null` for non-nullable fields. To update a single field use `ApiError(...).copyWith.fieldName(value)`.
  ///
  /// Example:
  /// ```dart
  /// ApiError(...).copyWith(id: 12, name: "My name")
  /// ```
  @override
  ApiError call({Object? error = const $CopyWithPlaceholder()}) {
    return ApiError(
      error: error == const $CopyWithPlaceholder() || error == null
          ? _value.error
          // ignore: cast_nullable_to_non_nullable
          : error as ApiErrorDetails,
    );
  }
}

extension $ApiErrorCopyWith on ApiError {
  /// Returns a callable class used to build a new instance with modified fields.
  /// Example: `instanceOfApiError.copyWith(...)` or `instanceOfApiError.copyWith.fieldName(...)`.
  // ignore: library_private_types_in_public_api
  _$ApiErrorCWProxy get copyWith => _$ApiErrorCWProxyImpl(this);
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ApiError _$ApiErrorFromJson(Map<String, dynamic> json) =>
    $checkedCreate('ApiError', json, ($checkedConvert) {
      $checkKeys(json, requiredKeys: const ['error']);
      final val = ApiError(
        error: $checkedConvert(
          'error',
          (v) => ApiErrorDetails.fromJson(v as Map<String, dynamic>),
        ),
      );
      return val;
    });

Map<String, dynamic> _$ApiErrorToJson(ApiError instance) => <String, dynamic>{
  'error': instance.error.toJson(),
};
