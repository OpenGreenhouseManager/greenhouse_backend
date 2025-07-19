# Error Handling Improvements Summary

## Overview

This document summarizes the comprehensive error handling improvements made to the API to address the issues where most errors were hidden behind generic 500 Internal Server Error responses, making debugging and client error handling difficult.

## Key Improvements

### 1. Standardized Error Response Format

- **Before**: Inconsistent error responses, mostly 500 errors with debug strings
- **After**: Consistent JSON format across all endpoints:

```json
{
  "error": "ERROR_TYPE",
  "message": "Human-readable error message", 
  "status_code": 400,
  "details": {
    "field": "Additional context (optional)"
  }
}
```

### 2. Proper HTTP Status Code Mapping

- **400 Bad Request**: Invalid client input
- **401 Unauthorized**: Authentication required/failed
- **403 Forbidden**: Access denied (authenticated but insufficient permissions)
- **404 Not Found**: Resource doesn't exist
- **409 Conflict**: Resource conflicts (e.g., username taken)
- **422 Unprocessable Entity**: Semantically incorrect requests
- **429 Too Many Requests**: Rate limiting
- **500 Internal Server Error**: Actual server errors (now rare)
- **502 Bad Gateway**: External service failures
- **503 Service Unavailable**: Temporary service issues

### 3. Security-Conscious Error Messages

- **Internal errors** are logged with full details but return generic user-safe messages
- **Database schema details** are hidden from API responses
- **Stack traces and debug information** are removed from client responses
- **Service architecture details** are abstracted away

### 4. Comprehensive Error Logging and Monitoring

- All internal errors are logged with structured tracing
- Errors are automatically sent to Sentry for monitoring
- User-facing messages are sanitized while preserving debugging information

## Implementation Details

### Core Error Module (`greenhouse_core/src/error.rs`)

Created a centralized error handling system with:

- `ApiErrorResponse` struct for consistent JSON responses
- `ErrorCategory` enum for standardized error types
- `IntoApiError` trait for converting domain errors to API responses
- Helper functions for common error scenarios
- Logging macro for automatic error reporting

### Service Layer Updates

Updated all service error handlers:

- **Auth Service**: Maps authentication/authorization errors to appropriate HTTP codes
- **Device Service**: Handles device connectivity and validation errors
- **Data Storage Service**: Manages data persistence and format errors

### API Layer Updates

Enhanced all API endpoint error handling:

- **Request validation errors** → 400 Bad Request
- **Authentication failures** → 401 Unauthorized
- **Resource not found** → 404 Not Found
- **Service unavailable** → 502/503 based on the issue type

## Benefits for Clients

### Before
```bash
# Most errors looked like this:
HTTP/1.1 500 Internal Server Error
Content-Type: text/plain

DatabaseConnection
```

### After
```bash
# Now errors are informative and actionable:
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "error": "UNAUTHORIZED",
  "message": "Invalid username or password",
  "status_code": 401
}
```

### Client-Side Error Handling

Clients can now implement proper error handling:

```typescript
function handleApiError(error: ApiError) {
  switch (error.error) {
    case 'UNAUTHORIZED':
      redirectToLogin();
      break;
    case 'NOT_FOUND':
      showNotFoundMessage();
      break;
    case 'INVALID_INPUT':
      highlightValidationErrors(error.details);
      break;
    case 'SERVICE_UNAVAILABLE':
      showRetryOption();
      break;
    default:
      showGenericError(error.message);
  }
}
```

## Security Considerations

### What We Hide
- Database connection strings and schemas
- Internal service URLs and architecture
- Stack traces and debug information
- File paths and system details
- Library/framework versions

### What We Expose
- Resource existence (404 vs 403 carefully chosen)
- Input validation errors (sanitized)
- Service availability status
- Rate limiting information

## Debugging Benefits

### For Developers
- **Structured logging** with detailed error context
- **Sentry integration** for error tracking and alerting
- **Request tracing** to identify error sources
- **Consistent error patterns** across services

### For DevOps
- **Meaningful error metrics** for monitoring
- **Service health indicators** from error types
- **Actionable alerts** based on error categories
- **Performance insights** from error patterns

## Testing

The new error handling can be tested with:

```bash
# Test authentication
curl -X POST /api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "invalid", "password": "wrong"}'
# Expected: 401 with UNAUTHORIZED error

# Test validation
curl -X POST /api/device \
  -H "Content-Type: application/json" \
  -d '{"invalid": "data"}'
# Expected: 400 with INVALID_INPUT error

# Test not found
curl /api/device/00000000-0000-0000-0000-000000000000
# Expected: 404 with NOT_FOUND error
```

## Migration Path

The changes are backward compatible in terms of functionality, but client applications should be updated to:

1. **Parse JSON error responses** instead of plain text
2. **Handle different HTTP status codes** appropriately
3. **Use the `error` field** for programmatic error handling
4. **Display the `message` field** to users
5. **Utilize the `details` field** for form validation

## Files Modified

- `greenhouse_core/src/error.rs` - New centralized error handling
- `greenhouse_core/src/lib.rs` - Export error module
- `services/*/src/router/error.rs` - Updated service error handlers
- `api/web/src/*/error.rs` - Updated API error handlers
- `api/web/src/device/service.rs` - Simplified with new error handling
- `docs/ERROR_HANDLING.md` - Comprehensive documentation

This improvement significantly enhances the API's usability, security, and maintainability while providing much better debugging capabilities for both developers and clients.