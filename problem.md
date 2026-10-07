# Goal

I want to write a character sheet builder in Rust for D&D 5e.

# The output

The character sheet we want to create has something like the following structure:

- Character Sheet
    - Race
        - Traits
            - Effects
    - Subrace (optional)
        - Traits
            - Effects
    - Class
        - Features
            - Effects
    - Subclass (optional)
        - Features
            - Effects
    - Background
        - Features
            - Effects

This is the final structure of the character sheet. It is a big nested structure. In the end it all boils down to
effects like adding a proficiency, condition, advantage or modeling certain interactive rules like rage, bardic
inspiration, spells etc. The program will find these effects and use them in the game engine.

But where do these race, class, background etc. definitions come from? They come from a bigger json: The input.

# The input

A large json file will be written. The file represents the D&D 5e SRD 5.1 document. It describes how to build a race,
class etc. These are our blueprints. You cannot get a Race out of this. You get a Blueprint that, after the user has
interacted with it and made all the choices, will become a Race. Same goes for the other components of the character
sheet.

The input, or the "source", effectively contains each form of a Dwarf, Elf, Human that can be made. Same goes for
Barbarian, Paladin, Wizard etc. and the backgrounds. The races are instances of the same type, but they're not instances
of the type Race because their choices have not been made.

# The problem

Let's take The Dwarf Race as an example. Most of the traits are in place, already built. Ability Score Increase, Speed,
Size, these are hardcoded values.

Only the tool proficiency trait needs a choice to be made by the user.

## What I was doing before...

Let's make a trait blueprint. The trait blueprint holds a template for the tool proficiency trait and the choices the
user can make (with a cardinality for when multiple choices can be made). The choices contain patches that should be
made to the template when that choice is made. But where do we put the trait blueprint?

We cannot put it in a race because a race only has traits, not trait blueprints. A very big requirement of this project
is that we have a water tight domain language. So where to put it? Let's make a race blueprint that contains both traits
(for the hardcoded traits) and trait blueprints (for the choices that need to be made). But where to put the Race
Blueprint?

...

This is the issue, adding one blueprint on the lowest level of the object tree has the advantage of easy to write
patches but forces us to write a lot of higher level boilerplate that becomes unmaintainable very quick.

The other solution is that we won't have blueprints on the lowest level of the hierarchy but, only templates and choices
separately on the highest level. Templates are incomplete versions of the final Race and choices contain patches that
should be made on the lowest level of the hierarchy.

We solve the unmaintainable boilerplate like this but writing patches will get very elaborated, very quickly to the
point it becomes not user-friendly. The users of this crate will be the ones writing their versions of their own
sources. They will use the API to construct their own types of sources like the SRD, PHB, DMG etc. They must be able to
use the API but when they use the API they won't be able to write incomplete versions to use as templates.

### Example

```rust
struct Tool {
    name: String,
    cost: usize,
}

enum Proficiency {
    Tool(Tool),
}
```

A user cannot write `Proficiency::Tool()` as a placeholder for the program to insert a tool once the final user has made
a choice.

# Requirements

1. Tight domain model. A Race has a name, a description and traits. A Race should never be able to contain a partial
   trait, trait blueprint or such.
2. No unmaintainable boilerplate unless we can soundly macro-generate it from the domain language.
3. Patches must be as small as possible, as close to their change as possible.
4. The API must be writable from Rust itself for ease of use and user-friendliness.
5. Type safety. Yes, we could describe everything as a hashmap of strings, but that's too error-prone. Use strong types. 


