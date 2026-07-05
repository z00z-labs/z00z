# Performance-Focused Code Review: [Feature/PR Name]

**Reviewed by:** [Your Name]
**Date:** [YYYY-MM-DD]
**Commit/PR:** [#123 or commit hash]
**Review Type:** Performance Review

---

## Executive Summary

**Performance Verdict:** ✅ OPTIMAL | ⚠️ NEEDS OPTIMIZATION | ❌ CRITICAL ISSUES

**Performance Risk:** 🟢 Low | 🟡 Medium | 🔴 High

**Overview:**
[Brief assessment of performance characteristics]

**Critical Issues:** X
**Optimization Opportunities:** Y

---

## Database Performance

### Query Analysis

**N+1 Query Detection:**
- [ ] No N+1 query patterns found
- [ ] Relationships properly eager-loaded
- [ ] Batch queries used where appropriate

**Findings:**
[List any N+1 issues with location and fix]

---

**Index Coverage:**
- [ ] All queried columns have indexes
- [ ] Foreign keys indexed
- [ ] Composite indexes for multi-column queries

**Missing Indexes:**
[List any missing indexes]

**Recommendation:**
```sql
CREATE INDEX idx_table_column ON table(column);
```

---

**Query Complexity:**
- [ ] No queries returning > 1000 rows without pagination
- [ ] JOINs limited to < 5 tables
- [ ] Subqueries optimized

**Slow Queries Found:**
[List queries that need optimization]

---

## Algorithm Complexity

### Time Complexity Analysis

**Function:** `[functionName]` at `file.ts:line`

**Current Complexity:** O(n²) | O(n log n) | O(n) | O(1)

**Analysis:**
```typescript
// Current implementation
[Code snippet]
```

**Issue:**
[Explanation of performance problem]

**Optimized Version:** O([complexity])
```typescript
// Optimized implementation
[Code snippet]
```

**Impact:**
- Small dataset (n < 100): [negligible | minor | significant]
- Medium dataset (n < 1000): [negligible | minor | significant]
- Large dataset (n > 1000): [negligible | minor | significant]

---

## Memory Usage

### Memory Leak Detection

**Potential Leaks:**
- [ ] No unbounded caches
- [ ] Event listeners cleaned up
- [ ] Closures don't hold large objects
- [ ] Timers/intervals cleared

**Issues Found:**
[List memory leak risks]

---

### Memory Allocation

**Large Object Analysis:**
- [ ] No unnecessary object cloning
- [ ] Streaming used for large data
- [ ] Pagination for large lists

**Findings:**
[List memory-heavy operations]

---

## Bundle Size Impact

### Frontend Bundle Analysis

**Files Modified:**
- `file1.tsx` - +X KB
- `file2.tsx` - +Y KB

**Total Impact:** +Z KB

**Bundle Budget:** [Current: X KB] [Budget: Y KB] [Status: ✅ Within | ⚠️ Near Limit | ❌ Exceeded]

**Optimization Opportunities:**
- [ ] Dynamic imports for code splitting
- [ ] Tree shaking enabled
- [ ] Remove unused dependencies
- [ ] Optimize images/assets

**Recommendations:**
```typescript
// Use dynamic imports
const HeavyComponent = lazy(() => import('./HeavyComponent'));
```

---

## Frontend Performance

### Rendering Performance

**React Component Analysis:**

**Component:** `[ComponentName]`

**Issues:**
- [ ] Unnecessary re-renders
- [ ] Missing React.memo
- [ ] Expensive calculations not memoized
- [ ] No virtualization for long lists

**Current:**
```typescript
// Current implementation
[Code snippet showing problem]
```

**Optimized:**
```typescript
// Optimized with useMemo/useCallback
[Code snippet showing fix]
```

---

### Network Performance

**API Calls:**
- [ ] No excessive API calls
- [ ] Request caching implemented
- [ ] Debouncing on user inputs
- [ ] Response compression enabled

**Findings:**
[List API performance issues]

---

### Asset Loading

**Images:**
- [ ] Images optimized (WebP format)
- [ ] Responsive images used
- [ ] Lazy loading implemented
- [ ] Proper sizing (no oversized images)

**Fonts:**
- [ ] Font loading optimized
- [ ] System fonts as fallback
- [ ] Font subsetting used

**JavaScript:**
- [ ] Code splitting implemented
- [ ] Critical JS inlined
- [ ] Non-critical JS deferred

**Findings:**
[List asset optimization opportunities]

---

## Load Testing Results

### Performance Metrics

**Before Changes:**
- **Response Time (p50):** X ms
- **Response Time (p95):** Y ms
- **Response Time (p99):** Z ms
- **Throughput:** X req/sec
- **Error Rate:** Y%

**After Changes (Projected):**
- **Response Time (p50):** X ms
- **Response Time (p95):** Y ms
- **Response Time (p99):** Z ms
- **Throughput:** X req/sec
- **Error Rate:** Y%

**Impact:** [Improved by X% | Degraded by Y% | No significant change]

---

## Lighthouse/Core Web Vitals

### Metrics

**First Contentful Paint (FCP):**
- Current: X s
- Target: < 1.8s
- Status: [✅ Pass | ⚠️ Needs Improvement | ❌ Fail]

**Largest Contentful Paint (LCP):**
- Current: X s
- Target: < 2.5s
- Status: [✅ Pass | ⚠️ Needs Improvement | ❌ Fail]

**Time to Interactive (TTI):**
- Current: X s
- Target: < 3.8s
- Status: [✅ Pass | ⚠️ Needs Improvement | ❌ Fail]

**Total Blocking Time (TBT):**
- Current: X ms
- Target: < 200ms
- Status: [✅ Pass | ⚠️ Needs Improvement | ❌ Fail]

**Cumulative Layout Shift (CLS):**
- Current: X
- Target: < 0.1
- Status: [✅ Pass | ⚠️ Needs Improvement | ❌ Fail]

---

## Overall Performance Assessment

### Critical Performance Issues 🔴

[List issues that will significantly degrade user experience]

### High Priority Optimizations 🟠

[List optimizations that will provide noticeable improvement]

### Medium Priority Optimizations 🟡

[List nice-to-have optimizations]

---

## Recommendations

### Immediate Actions (Before Merge)
1. [Fix with estimated time]
2. [Fix with estimated time]

### Short-term (Within Sprint)
1. [Optimization]
2. [Optimization]

### Long-term (Technical Debt)
1. [Refactoring opportunity]
2. [Architecture improvement]

**Total Optimization Time:** X hours

---

## Performance Testing Plan

**Before Merge:**
- [ ] Load test with [X] concurrent users
- [ ] Memory profiling
- [ ] Bundle size analysis
- [ ] Lighthouse audit

**After Deployment:**
- [ ] Monitor response times
- [ ] Watch for memory leaks
- [ ] Track Core Web Vitals
- [ ] Set up alerts

---

## Performance Sign-off

**Performance Status:** [✅ APPROVED | ⚠️ CONDITIONAL APPROVAL | ❌ REQUIRES OPTIMIZATION]

**Conditions (if conditional):**
- [List performance improvements required]

**Monitoring Plan:**
- [What metrics to monitor post-deployment]

**Reviewer:** [Your Name]
**Date:** [YYYY-MM-DD]
