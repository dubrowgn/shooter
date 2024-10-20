# Ability

## Terms

* Blizzard Trigger-esque
    * Event (a.k.a trigger) - what events can I register for?
        * tag added/removed
        * attr changed/added/removed
    * Action - what API's can I call?
        * gfx effect
        * sound effect
        * add/remove/modify attributes
        * add/remove modifiers
        * add/remove tags
    * Condition - filtering events to something more specific (e.g. a specific unit)

* Unreal GAS
    * Status (a.k.a. tag) - effectively a boolean set in a bit mask (e.g. "stunned")
    * Attribute (a.k.a value) - a value (e.g. health, charges, ammo, damage)
        * base value
        * current value - base + modifiers
        * ? min/max values?
        * potentially attr dependency graphs
    * Modifier - change an attribute's current value ()
        * add
        * multiply
        * divide? (could just multiply)
        * override
        - order is important; add first, override last
        - do scalars multiply together or add? (+50%, +50% => 200% or 225%?)
    * Effect - do something
        * instant - just execute
        * duration - add/remove/tick actions; automatic exiration
            * e.g. add/remove modifier
        * infinite - add/remove/tick actions; manual removal
    * Ability - "actor" action or skill
        - jump, shoot, passives, open door, collect resource, building
    * Cue - non-gameplay effects (sounds, gfx, camera)

* Unified?
    * status
    * value
    * modifier
    * effect
    * ability
    * cue

## References

* https://github.com/tranek/GASDocumentation
