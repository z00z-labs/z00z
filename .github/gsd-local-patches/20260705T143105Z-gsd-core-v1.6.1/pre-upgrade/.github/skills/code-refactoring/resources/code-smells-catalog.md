# Code Smells Catalog

**Based on Martin Fowler's "Refactoring" (2nd Edition) and Refactoring.guru**

A comprehensive catalog of 21 code smells organized into 5 categories, with detection patterns and recommended refactorings.

---

## 📚 Table of Contents

1. [Bloaters](#bloaters) - Growing code that's hard to manage
2. [Object-Orientation Abusers](#object-orientation-abusers) - Incorrect OO implementation
3. [Change Preventers](#change-preventers) - Changes require many other changes
4. [Dispensables](#dispensables) - Pointless and unnecessary code
5. [Couplers](#couplers) - Excessive coupling between classes

---

## 🔴 Bloaters

**Definition:** Code, methods, and classes that have grown so large they're hard to work with.

### 1. Long Method

**Smell:** Method/function exceeds reasonable length.

**Detection Patterns:**
- JavaScript/TypeScript: Function >50 lines
- Python: Function >50 lines
- React: Component render method >30 lines
- Any language: Method requires scrolling to understand

**Why It's Bad:**
- Hard to understand
- Difficult to test
- Contains hidden bugs
- Violates Single Responsibility Principle

**Recommended Refactoring:**
- **Extract Method:** Break into smaller methods
- **Replace Temp with Query:** Remove temporary variables
- **Introduce Parameter Object:** Group related parameters
- **Preserve Whole Object:** Pass entire object instead of fields

**Example (JavaScript):**
```javascript
// ❌ BAD: Long method (100+ lines)
function processOrder(order) {
  // 20 lines of validation
  if (!order.customer) throw new Error("No customer");
  if (!order.items || order.items.length === 0) throw new Error("No items");
  // ... more validation

  // 30 lines of price calculation
  let subtotal = 0;
  for (let item of order.items) {
    subtotal += item.price * item.quantity;
  }
  // ... more calculation

  // 30 lines of shipping logic
  let shipping = 0;
  if (order.customer.country === "US") {
    shipping = subtotal > 100 ? 0 : 10;
  }
  // ... more shipping logic

  // 20 lines of payment processing
  // ...
}

// ✅ GOOD: Extracted methods
function processOrder(order) {
  validateOrder(order);
  const subtotal = calculateSubtotal(order.items);
  const shipping = calculateShipping(order, subtotal);
  const tax = calculateTax(subtotal, order.customer);
  const total = subtotal + shipping + tax;
  return processPayment(order, total);
}

function validateOrder(order) {
  if (!order.customer) throw new Error("No customer");
  if (!order.items || order.items.length === 0) throw new Error("No items");
}

function calculateSubtotal(items) {
  return items.reduce((sum, item) => sum + item.price * item.quantity, 0);
}

function calculateShipping(order, subtotal) {
  if (order.customer.country === "US") {
    return subtotal > 100 ? 0 : 10;
  }
  // ... other shipping logic
}
```

---

### 2. Large Class

**Smell:** Class has too many instance variables or methods.

**Detection Patterns:**
- JavaScript/TypeScript: Class >200 lines
- Python: Class >300 lines
- More than 10-15 methods
- More than 10 instance variables
- Class name includes "Manager", "Controller", or "Util" (often God classes)

**Why It's Bad:**
- Violates Single Responsibility Principle
- Hard to understand and maintain
- High coupling
- Difficult to test

**Recommended Refactoring:**
- **Extract Class:** Split into multiple focused classes
- **Extract Subclass:** For parts used in special cases
- **Extract Interface:** If class has multiple responsibilities
- **Replace Data Value with Object:** For groups of related data

**Example (Python):**
```python
# ❌ BAD: God class (400+ lines)
class UserManager:
    def __init__(self):
        self.db_connection = None
        self.cache = None
        self.email_service = None
        self.analytics = None
        # ... 10 more instance variables

    def create_user(self): ...       # 50 lines
    def update_user(self): ...       # 40 lines
    def delete_user(self): ...       # 30 lines
    def authenticate(self): ...      # 60 lines
    def send_welcome_email(self): ...  # 40 lines
    def track_user_action(self): ...   # 50 lines
    def generate_report(self): ...     # 80 lines
    # ... 10 more methods

# ✅ GOOD: Split into focused classes
class UserRepository:
    """Handles user persistence"""
    def create(self, user): ...
    def update(self, user): ...
    def delete(self, user_id): ...
    def find_by_id(self, user_id): ...

class UserAuthenticator:
    """Handles authentication"""
    def authenticate(self, credentials): ...
    def validate_token(self, token): ...

class UserNotifier:
    """Handles user notifications"""
    def send_welcome_email(self, user): ...
    def send_password_reset(self, user): ...

class UserAnalytics:
    """Handles user analytics"""
    def track_action(self, user, action): ...
    def generate_report(self, user_id): ...

class UserService:
    """Orchestrates user operations"""
    def __init__(self):
        self.repository = UserRepository()
        self.authenticator = UserAuthenticator()
        self.notifier = UserNotifier()
        self.analytics = UserAnalytics()
```

---

### 3. Primitive Obsession

**Smell:** Using primitives instead of small objects for simple tasks.

**Detection Patterns:**
- Strings/numbers used for complex data (phone numbers, currencies, addresses)
- Constants used instead of enums
- Type codes or flags instead of classes
- Array indices used for field names

**Why It's Bad:**
- No type safety
- No validation
- No associated behavior
- Hard to change format

**Recommended Refactoring:**
- **Replace Data Value with Object**
- **Introduce Parameter Object**
- **Replace Type Code with Class/Subclasses**

**Example (TypeScript):**
```typescript
// ❌ BAD: Primitives for complex data
function createInvoice(
  customerName: string,
  customerEmail: string,
  customerAddress: string,
  customerPhone: string,
  amount: number,
  currency: string,
  dueDate: string
) {
  // Validation scattered everywhere
  if (!customerEmail.includes("@")) throw new Error("Invalid email");
  if (currency !== "USD" && currency !== "EUR") throw new Error("Invalid currency");
  // ...
}

// ✅ GOOD: Value objects for complex data
class Email {
  constructor(private value: string) {
    if (!value.includes("@")) throw new Error("Invalid email");
  }
  toString() { return this.value; }
}

class Money {
  constructor(
    private amount: number,
    private currency: "USD" | "EUR" | "GBP"
  ) {}

  add(other: Money): Money {
    if (this.currency !== other.currency) {
      throw new Error("Cannot add different currencies");
    }
    return new Money(this.amount + other.amount, this.currency);
  }
}

class Customer {
  constructor(
    private name: string,
    private email: Email,
    private address: Address,
    private phone: PhoneNumber
  ) {}
}

function createInvoice(
  customer: Customer,
  amount: Money,
  dueDate: Date
) {
  // All validation happens in value object constructors
  // Type-safe, reusable, testable
}
```

---

### 4. Long Parameter List

**Smell:** Method has more than 3-4 parameters.

**Detection Patterns:**
- Function/method with >5 parameters
- Parameters frequently change together
- Same parameters passed to multiple methods
- Primitive types where object would be better

**Why It's Bad:**
- Hard to remember parameter order
- Easy to make mistakes
- Hard to change
- Often indicates missing abstraction

**Recommended Refactoring:**
- **Introduce Parameter Object**
- **Preserve Whole Object**
- **Replace Parameter with Method Call**

**Example (JavaScript):**
```javascript
// ❌ BAD: Too many parameters
function createUser(
  firstName,
  lastName,
  email,
  phone,
  street,
  city,
  state,
  zipCode,
  country,
  isActive,
  role
) {
  // ...
}

createUser(
  "John",
  "Doe",
  "john@example.com",
  "555-1234",
  "123 Main St",
  "Springfield",
  "IL",
  "62701",
  "USA",
  true,
  "admin"
); // Easy to mix up order!

// ✅ GOOD: Parameter object
class UserParams {
  constructor(data) {
    this.name = { first: data.firstName, last: data.lastName };
    this.contact = { email: data.email, phone: data.phone };
    this.address = {
      street: data.street,
      city: data.city,
      state: data.state,
      zipCode: data.zipCode,
      country: data.country
    };
    this.isActive = data.isActive;
    this.role = data.role;
  }
}

function createUser(params) {
  // Much clearer, easier to extend
}

createUser(new UserParams({
  firstName: "John",
  lastName: "Doe",
  email: "john@example.com",
  phone: "555-1234",
  street: "123 Main St",
  city: "Springfield",
  state: "IL",
  zipCode: "62701",
  country: "USA",
  isActive: true,
  role: "admin"
}));
```

---

### 5. Data Clumps

**Smell:** Same group of data items appears together in multiple places.

**Detection Patterns:**
- Same 3+ parameters passed together to multiple methods
- Same fields in multiple classes
- Deleting one item makes others meaningless

**Why It's Bad:**
- Duplication
- Hard to change
- Missing abstraction
- Scattered validation

**Recommended Refactoring:**
- **Extract Class** for the clump
- **Introduce Parameter Object**
- **Preserve Whole Object**

**Example (Python):**
```python
# ❌ BAD: Data clumps
def calculate_distance(x1, y1, z1, x2, y2, z2):
    return math.sqrt((x2-x1)**2 + (y2-y1)**2 + (z2-z1)**2)

def midpoint(x1, y1, z1, x2, y2, z2):
    return ((x1+x2)/2, (y1+y2)/2, (z1+z2)/2)

def draw_line(x1, y1, z1, x2, y2, z2, color):
    # ... x1, y1, z1 and x2, y2, z2 always appear together!

# ✅ GOOD: Extract class
class Point3D:
    def __init__(self, x, y, z):
        self.x = x
        self.y = y
        self.z = z

    def distance_to(self, other):
        return math.sqrt(
            (other.x - self.x)**2 +
            (other.y - self.y)**2 +
            (other.z - self.z)**2
        )

    def midpoint_to(self, other):
        return Point3D(
            (self.x + other.x) / 2,
            (self.y + other.y) / 2,
            (self.z + other.z) / 2
        )

def draw_line(start: Point3D, end: Point3D, color):
    # Much clearer!
```

---

## 🟠 Object-Orientation Abusers

**Definition:** Incomplete or incorrect application of object-oriented principles.

### 6. Switch Statements (Type Codes)

**Smell:** Complex switch/case or if-else chains based on object type.

**Detection Patterns:**
- Switch on type code or class type
- Same switch appears in multiple places
- New types require modifying switch statements
- switch/if-else >5 branches

**Why It's Bad:**
- Violates Open/Closed Principle
- Code duplication
- Hard to extend
- Easy to forget updating all switches

**Recommended Refactoring:**
- **Replace Type Code with Subclasses/State/Strategy**
- **Replace Conditional with Polymorphism**
- **Use dispatch table/map**

**Example (TypeScript):**
```typescript
// ❌ BAD: Switch statement scattered everywhere
function getSpeed(vehicle) {
  switch (vehicle.type) {
    case "car":
      return vehicle.engine * 2;
    case "plane":
      return vehicle.engine * 10;
    case "boat":
      return vehicle.engine * 1.5;
    default:
      return 0;
  }
}

function getFuelConsumption(vehicle) {
  switch (vehicle.type) {
    case "car":
      return vehicle.engine / 10;
    case "plane":
      return vehicle.engine / 5;
    case "boat":
      return vehicle.engine / 8;
    default:
      return 0;
  }
}

// ✅ GOOD: Polymorphism
abstract class Vehicle {
  constructor(protected engine: number) {}
  abstract getSpeed(): number;
  abstract getFuelConsumption(): number;
}

class Car extends Vehicle {
  getSpeed() { return this.engine * 2; }
  getFuelConsumption() { return this.engine / 10; }
}

class Plane extends Vehicle {
  getSpeed() { return this.engine * 10; }
  getFuelConsumption() { return this.engine / 5; }
}

class Boat extends Vehicle {
  getSpeed() { return this.engine * 1.5; }
  getFuelConsumption() { return this.engine / 8; }
}

// Adding new vehicle type = new class, no changes to existing code!
```

---

### 7. Temporary Field

**Smell:** Object has fields that are only set under certain circumstances.

**Detection Patterns:**
- Fields that are null/undefined most of the time
- Fields only used in specific algorithms
- Complex conditionals checking if field is set

**Why It's Bad:**
- Confusing - why is this field here?
- Hard to understand object state
- Wastes memory
- Often indicates missing class

**Recommended Refactoring:**
- **Extract Class** for temporary fields
- **Introduce Null Object**
- **Replace Method with Method Object**

---

### 8. Refused Bequest

**Smell:** Subclass doesn't use most of inherited methods/fields.

**Detection Patterns:**
- Subclass throws "not supported" exceptions
- Subclass doesn't use parent's methods
- Inheritance used for code reuse, not "is-a" relationship

**Why It's Bad:**
- Wrong hierarchy
- Violates Liskov Substitution Principle
- Confusing design

**Recommended Refactoring:**
- **Replace Inheritance with Delegation**
- **Create New Sibling Class**
- **Remove subclass if unnecessary**

---

### 9. Alternative Classes with Different Interfaces

**Smell:** Two classes do the same thing but have different method names.

**Detection Patterns:**
- Similar algorithms with different method names
- Code duplication between classes
- Switching between classes requires rewriting

**Why It's Bad:**
- Duplication
- Inconsistency
- Hard to switch implementations

**Recommended Refactoring:**
- **Rename Method** to make consistent
- **Extract Superclass**
- **Extract Interface**

---

## 🟡 Change Preventers

**Definition:** Changes in one place require many changes elsewhere.

### 10. Divergent Change

**Smell:** One class is commonly changed in different ways for different reasons.

**Detection Patterns:**
- "Whenever we add a new database, we change these 3 methods"
- "Whenever we add UI field, we change these 5 methods"
- One class modified for multiple types of changes

**Why It's Bad:**
- Violates Single Responsibility
- High change risk
- Hard to predict impact

**Recommended Refactoring:**
- **Extract Class** for each type of change
- **Move Method** to appropriate class

---

### 11. Shotgun Surgery

**Smell:** Every change requires tiny changes in many classes.

**Detection Patterns:**
- Simple change touches 5+ classes
- Adding feature requires editing scattered methods
- Changes ripple through system

**Why It's Bad:**
- Error-prone
- Time-consuming
- Easy to miss a change

**Recommended Refactoring:**
- **Move Method/Field** to centralize
- **Inline Class** if too fragmented

---

### 12. Parallel Inheritance Hierarchies

**Smell:** When you add subclass to one hierarchy, you need to add subclass to another.

**Detection Patterns:**
- Class name prefixes match in two hierarchies
- Every time you subclass A, you subclass B
- Two hierarchies grow together

**Why It's Bad:**
- Duplication
- Hard to maintain
- Easy to forget

**Recommended Refactoring:**
- **Move Method/Field** to eliminate one hierarchy
- **Collapse Hierarchy**

---

## 🟢 Dispensables

**Definition:** Code that adds no value and can be removed.

### 13. Comments (as Code Smell)

**Smell:** Comment explaining what code does (not why).

**Detection Patterns:**
- Comment repeats what code says
- Comment explains complex code
- TODO comments left for years
- Commented-out code

**Why It's Bad:**
- Code should be self-explanatory
- Comments get outdated
- Indicates unclear code

**Recommended Refactoring:**
- **Extract Method** with descriptive name
- **Rename Method** to clarify
- **Introduce Assertion** instead of assumption comment
- Delete commented-out code (use version control!)

**Example:**
```javascript
// ❌ BAD: Comment explaining what code does
// Check if user is eligible for discount
if (user.orders > 10 && user.totalSpent > 1000 && user.membershipYears > 2) {
  // Apply 20% discount
  price = price * 0.8;
}

// ✅ GOOD: Self-explanatory code
if (user.isEligibleForLoyaltyDiscount()) {
  price = user.applyLoyaltyDiscount(price);
}

class User {
  isEligibleForLoyaltyDiscount() {
    return this.orders > 10 &&
           this.totalSpent > 1000 &&
           this.membershipYears > 2;
  }

  applyLoyaltyDiscount(price) {
    return price * 0.8;
  }
}
```

---

### 14. Duplicate Code

**Smell:** Same code structure in multiple places.

**Detection Patterns:**
- Exact code duplication
- Similar code with slight variations
- Same algorithm in different methods

**Why It's Bad:**
- Maintenance nightmare
- Bug appears in multiple places
- Changes must be replicated

**Recommended Refactoring:**
- **Extract Method**
- **Pull Up Method** (if in subclasses)
- **Form Template Method**

---

### 15. Lazy Class

**Smell:** Class doesn't do enough to justify existence.

**Detection Patterns:**
- Class with only 1-2 methods
- Class that just delegates to another
- Empty or near-empty class
- Class used in only one place

**Why It's Bad:**
- Unnecessary complexity
- Hard to navigate codebase
- Over-engineering

**Recommended Refactoring:**
- **Inline Class**
- **Collapse Hierarchy** if subclass
- Delete if truly unused

---

### 16. Data Class

**Smell:** Class with only fields and getters/setters, no behavior.

**Detection Patterns:**
- Only public fields or getters/setters
- No logic methods
- Other classes manipulate its data

**Why It's Bad:**
- Violates encapsulation
- Behavior scattered in other classes
- Anemic domain model

**Recommended Refactoring:**
- **Move Method** behavior to data class
- **Encapsulate Field**
- **Remove Setting Method** for fields that shouldn't change

---

### 17. Dead Code

**Smell:** Code that's never executed.

**Detection Patterns:**
- Unused functions/methods
- Unreachable if/else branches
- Unused parameters
- Unused variables

**Why It's Bad:**
- Confusing
- Wastes maintenance effort
- Can't be refactored safely

**Recommended Refactoring:**
- **Delete it!** (version control remembers)

---

### 18. Speculative Generality

**Smell:** "We might need this someday" code that's never used.

**Detection Patterns:**
- Abstract classes with one subclass
- Unused parameters
- Overly complex "flexible" solutions
- Features no one asked for

**Why It's Bad:**
- YAGNI violation (You Aren't Gonna Need It)
- Harder to understand
- Wastes time

**Recommended Refactoring:**
- **Collapse Hierarchy**
- **Inline Class**
- **Remove Parameter**
- Simplify!

---

## 🔵 Couplers

**Definition:** Excessive coupling between classes.

### 19. Feature Envy

**Smell:** Method uses data from another class more than its own.

**Detection Patterns:**
- Method calls other class's getters extensively
- Method logic primarily about another object
- Method would make more sense in another class

**Why It's Bad:**
- Wrong responsibility
- High coupling
- Hard to change

**Recommended Refactoring:**
- **Move Method** to envied class
- **Extract Method** if only part envies

**Example:**
```javascript
// ❌ BAD: Feature envy
class ShoppingCart {
  calculateTotal() {
    let total = 0;
    for (let item of this.items) {
      // Envies Item class!
      total += item.getPrice() * item.getQuantity();
      if (item.getDiscount()) {
        total -= item.getPrice() * item.getDiscount();
      }
    }
    return total;
  }
}

// ✅ GOOD: Move method to envied class
class Item {
  getTotalPrice() {
    let price = this.price * this.quantity;
    if (this.discount) {
      price -= this.price * this.discount;
    }
    return price;
  }
}

class ShoppingCart {
  calculateTotal() {
    return this.items.reduce((sum, item) => sum + item.getTotalPrice(), 0);
  }
}
```

---

### 20. Inappropriate Intimacy

**Smell:** Classes are too familiar with each other's private parts.

**Detection Patterns:**
- Class accesses private fields of another
- Classes spend too much time together
- Bidirectional dependencies

**Why It's Bad:**
- High coupling
- Hard to change
- Violates encapsulation

**Recommended Refactoring:**
- **Move Method/Field**
- **Extract Class** for common interests
- **Hide Delegate** or **Replace Delegation with Inheritance**

---

### 21. Message Chains

**Smell:** Code like `a.getB().getC().getD().doSomething()` (Law of Demeter violation).

**Detection Patterns:**
- Long chains of method calls
- Navigating through object structure
- Client knows too much about implementation

**Why It's Bad:**
- Brittle - change anywhere breaks everything
- High coupling
- Hard to test

**Recommended Refactoring:**
- **Hide Delegate**
- **Extract Method** to hide chain
- Move method closer to data

**Example:**
```javascript
// ❌ BAD: Message chain
const streetName = person.getAddress().getStreet().getName();

// ✅ GOOD: Hide delegate
class Person {
  getStreetName() {
    return this.address.getStreet().getName();
  }
}

const streetName = person.getStreetName();
```

---

## 🎯 Using This Catalog

### In Code Reviews
Look for these smells systematically:
1. Check file size (Bloater indicator)
2. Check method/function length
3. Look for repeated code patterns
4. Check parameter lists
5. Look for type codes and switches

### Automated Detection
Many tools can detect these:
- **SonarLint** - detects cognitive complexity, code smells
- **ESLint/TSLint** - configurable rules for many smells
- **Pylint** - Python code analysis
- **CodeScene** - AI-powered smell detection

### Priority
Not all smells are equal:

**Fix Immediately:**
- Duplicate code
- Long methods (>100 lines)
- Large classes (>500 lines)
- Switch statements on type

**Fix Soon:**
- Long parameter lists
- Feature envy
- Inappropriate intimacy
- Data clumps

**Fix When Touching Code:**
- Comments (as you refactor)
- Speculative generality
- Dead code
- Lazy classes

---

## 📚 Additional Resources

- **Fowler, Martin.** *Refactoring: Improving the Design of Existing Code (2nd Edition)*
- **Refactoring.guru** - Interactive catalog with examples
- **Clean Code by Robert Martin** - Related concepts

---

**Note:** This catalog is based on authoritative sources but adapted for practical use with modern languages (JavaScript/TypeScript/Python). Use it alongside the size-based triggers in SKILL.md for comprehensive refactoring guidance.

**Last Updated:** October 30, 2025
