
use super::*;

#[test]
fn test_score_empty_dockerfile() {
    let source = "";
    let score = score_dockerfile(source).unwrap();
    assert_eq!(score.grade, "F");
    assert!(score.score < 6.0);
}

#[test]
fn test_score_good_dockerfile() {
    let source = r#"FROM alpine:3.18

RUN set -euo pipefail && \
apk add --no-cache curl=8.2.1-r0 && \
rm -rf /var/cache/apk/*

USER nobody
"#;
    let score = score_dockerfile(source).unwrap();
    assert!(score.score >= 7.0, "Good Dockerfile should score >= 7.0");
    assert!(score.safety >= 7.0);
    assert!(score.determinism >= 7.0);
}

#[test]
fn test_score_excellent_dockerfile() {
    let source = r#"FROM alpine:3.18 AS builder

RUN set -euo pipefail && \
apk add --no-cache \
    curl=8.2.1-r0 \
    bash=5.2.15-r5 && \
rm -rf /var/cache/apk/*

FROM alpine:3.18

RUN set -euo pipefail && \
apk add --no-cache ca-certificates=20230506-r0 && \
rm -rf /var/cache/apk/* && \
adduser -D appuser

USER appuser
"#;
    let score = score_dockerfile(source).unwrap();
    assert!(
        score.score >= 8.0,
        "Excellent Dockerfile should score >= 8.0"
    );
    assert!(matches!(score.grade.as_str(), "A" | "A+" | "B" | "B+"));
}

#[test]
fn test_score_bad_dockerfile() {
    let source = r#"FROM alpine

RUN apk update
RUN apk upgrade
RUN apk add curl

CMD /app.sh
"#;
    let score = score_dockerfile(source).unwrap();
    assert!(score.score < 6.0, "Bad Dockerfile should score < 6.0");
    assert!(matches!(score.grade.as_str(), "D" | "F"));
    assert!(!score.suggestions.is_empty());
}

#[test]
fn test_detects_pipefail() {
    let with_pipefail = r#"FROM alpine:3.18
RUN set -euo pipefail && apk add curl
"#;
    let without_pipefail = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let score_with = score_dockerfile(with_pipefail).unwrap();
    let score_without = score_dockerfile(without_pipefail).unwrap();

    assert!(score_with.safety > score_without.safety);
}

#[test]
fn test_detects_version_pinning() {
    let pinned = r#"FROM alpine:3.18
RUN apk add curl=8.2.1-r0
"#;
    let unpinned = r#"FROM alpine:latest
RUN apk add curl
"#;

    let score_pinned = score_dockerfile(pinned).unwrap();
    let score_unpinned = score_dockerfile(unpinned).unwrap();

    assert!(score_pinned.determinism > score_unpinned.determinism);
}

#[test]
fn test_detects_cache_cleanup() {
    let with_cleanup = r#"FROM alpine:3.18
RUN apk add --no-cache curl && rm -rf /var/cache/apk/*
"#;
    let without_cleanup = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let score_with = score_dockerfile(with_cleanup).unwrap();
    let score_without = score_dockerfile(without_cleanup).unwrap();

    assert!(score_with.layer_optimization > score_without.layer_optimization);
}

#[test]
fn test_detects_user_directive() {
    let with_user = r#"FROM alpine:3.18
RUN adduser -D appuser
USER appuser
"#;
    let without_user = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let score_with = score_dockerfile(with_user).unwrap();
    let score_without = score_dockerfile(without_user).unwrap();

    assert!(score_with.security > score_without.security);
}

#[test]
fn test_multistage_build_bonus() {
    let multistage = r#"FROM alpine:3.18 AS builder
RUN apk add curl
FROM alpine:3.18
COPY --from=builder /app /app
"#;
    let single_stage = r#"FROM alpine:3.18
RUN apk add curl
"#;

    let score_multi = score_dockerfile(multistage).unwrap();
    let score_single = score_dockerfile(single_stage).unwrap();

    assert!(score_multi.layer_optimization >= score_single.layer_optimization);
}

// Issue #13: FROM scratch images should not be penalized for missing USER directive
#[test]
fn test_ISSUE_13_scratch_image_security_score() {
    let scratch_dockerfile = r#"FROM scratch
COPY --from=builder /build/binary /binary
ENTRYPOINT ["/binary"]
"#;
    let score = score_dockerfile(scratch_dockerfile).unwrap();

    // Scratch images should get high security score (no OS layer = minimal attack surface)
    assert!(
        score.security >= 8.0,
        "FROM scratch should score >= 8.0 security (got {})",
        score.security
    );

    // Should NOT suggest adding USER directive for scratch images
    let has_user_suggestion = score
        .suggestions
        .iter()
        .any(|s| s.contains("USER") || s.contains("non-root"));

    assert!(
        !has_user_suggestion,
        "FROM scratch should not suggest USER directive"
    );
}

#[test]
fn test_ISSUE_13_multistage_scratch_final_stage() {
    let multistage_scratch = r#"FROM alpine:3.18 AS builder
RUN apk add --no-cache curl && \
curl -o /binary https://example.com/binary

FROM scratch
COPY --from=builder /binary /binary
ENTRYPOINT ["/binary"]
"#;
    let score = score_dockerfile(multistage_scratch).unwrap();

    // Final stage is scratch - should have high security score
    assert!(
        score.security >= 8.0,
        "Multi-stage with FROM scratch final should score >= 8.0 security (got {})",
        score.security
    );

    // Should NOT suggest USER directive
    let has_user_suggestion = score
        .suggestions
        .iter()
        .any(|s| s.contains("USER") || s.contains("non-root"));

    assert!(
        !has_user_suggestion,
        "Multi-stage scratch should not suggest USER directive"
    );
}

#[test]
fn test_ISSUE_13_regular_image_still_requires_user() {
    let regular_dockerfile = r#"FROM alpine:3.18
RUN apk add curl
CMD ["/app"]
"#;
    let score = score_dockerfile(regular_dockerfile).unwrap();

    // Regular images should still be penalized for missing USER
    assert!(
        score.security < 8.0,
        "Regular image without USER should score < 8.0 security (got {})",
        score.security
    );

    // Should suggest USER directive for regular images
    let has_user_suggestion = score
        .suggestions
        .iter()
        .any(|s| s.contains("USER") || s.contains("non-root"));

    assert!(
        has_user_suggestion,
        "Regular image should suggest USER directive"
    );
}
