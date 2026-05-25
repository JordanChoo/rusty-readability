use std::collections::HashMap;
use std::sync::LazyLock;

static EXCEPTIONS: LazyLock<HashMap<&'static str, u8>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    let entries: &[(&str, u8)] = &[
        ("above", 2), ("again", 2), ("against", 2), ("aisle", 1),
        ("although", 2), ("among", 2), ("angel", 2), ("answer", 2),
        ("area", 3), ("army", 2), ("avenue", 3), ("awake", 2),
        ("aware", 2), ("beautiful", 3), ("because", 2), ("become", 2),
        ("before", 2), ("believe", 2), ("below", 2), ("between", 2),
        ("bicycle", 3), ("blouse", 1), ("bored", 1), ("breathe", 1),
        ("bridge", 1), ("browse", 1), ("bruise", 1), ("built", 1),
        ("bureau", 2), ("business", 2), ("cafe", 2), ("candle", 2),
        ("capable", 3), ("castle", 2), ("cause", 1), ("certain", 2),
        ("change", 1), ("chocolate", 3), ("choose", 1), ("circle", 2),
        ("climbed", 1), ("clothes", 1), ("colonel", 2), ("comfortable", 3),
        ("come", 1), ("continue", 3), ("control", 2), ("couple", 2),
        ("course", 1), ("coyote", 3), ("create", 2), ("creature", 2),
        ("cruise", 1), ("cycle", 2), ("dance", 1), ("danger", 2),
        ("daughter", 2), ("decide", 2), ("desire", 2), ("determine", 3),
        ("different", 2), ("dinosaur", 3), ("does", 1), ("done", 1),
        ("double", 2), ("eagle", 2), ("early", 2), ("engine", 2),
        ("enough", 2), ("every", 2), ("everyone", 3), ("everywhere", 3),
        ("example", 3), ("exchange", 2), ("excite", 2), ("exercise", 3),
        ("experience", 4), ("extreme", 2), ("eye", 1), ("false", 1),
        ("family", 3), ("favorite", 3), ("feature", 2), ("fierce", 1),
        ("figure", 2), ("fire", 1), ("fixed", 1), ("flour", 1),
        ("force", 1), ("foreign", 2), ("forever", 3), ("fortune", 2),
        ("friend", 1), ("garage", 2), ("general", 3), ("gentle", 2),
        ("genuine", 3), ("give", 1), ("glove", 1), ("gone", 1),
        ("gorgeous", 2), ("gotten", 2), ("gourmet", 2), ("graduate", 3),
        ("guide", 1), ("guinea", 2), ("have", 1), ("here", 1),
        ("honest", 2), ("honor", 2), ("horrible", 3), ("hour", 1),
        ("house", 1), ("huge", 1), ("hundred", 2), ("imagine", 3),
        ("improve", 2), ("include", 2), ("increase", 2), ("individual", 5),
        ("inspire", 2), ("interest", 3), ("involve", 2), ("island", 2),
        ("issue", 2), ("judge", 1), ("juice", 1), ("jungle", 2),
        ("knowledge", 2), ("language", 2), ("large", 1), ("laughed", 1),
        ("league", 1), ("library", 3), ("license", 2), ("little", 2),
        ("live", 1), ("lived", 1), ("loose", 1), ("lose", 1),
        ("love", 1), ("lovely", 2), ("machine", 2), ("manage", 2),
        ("marriage", 2), ("measure", 2), ("medicine", 3), ("menace", 2),
        ("merge", 1), ("message", 2), ("middle", 2), ("minute", 2),
        ("miracle", 3), ("miserable", 4), ("moisture", 2), ("molecule", 3),
        ("money", 2), ("mortgage", 2), ("mountain", 2), ("movie", 2),
        ("muscle", 2), ("natural", 3), ("nature", 2), ("nerve", 1),
        ("none", 1), ("notice", 2), ("novel", 2), ("nurse", 1),
        ("oblige", 2), ("ocean", 2), ("office", 2), ("once", 1),
        ("onion", 2), ("only", 2), ("operate", 3), ("opposite", 3),
        ("orange", 2), ("organize", 3), ("other", 2), ("oven", 2),
        ("over", 2), ("own", 1), ("palace", 2), ("pause", 1),
        ("peace", 1), ("people", 2), ("perhaps", 2), ("period", 3),
        ("perspire", 2), ("phrase", 1), ("piece", 1), ("pirate", 2),
        ("place", 1), ("please", 1), ("pledge", 1), ("plunge", 1),
        ("poem", 2), ("police", 2), ("possible", 3), ("postpone", 2),
        ("practice", 2), ("prepare", 2), ("presence", 2), ("pressure", 2),
        ("pretzel", 2), ("prince", 1), ("principle", 3), ("probably", 3),
        ("problem", 2), ("produce", 2), ("promise", 2), ("prove", 1),
        ("provide", 2), ("purchase", 2), ("purple", 2), ("purpose", 2),
        ("pursue", 2), ("puzzle", 2), ("quarter", 2), ("question", 2),
        ("queue", 1), ("quite", 1), ("range", 1), ("realize", 3),
        ("reason", 2), ("recipe", 3), ("recognize", 3), ("reduce", 2),
        ("release", 2), ("require", 2), ("resource", 2), ("restaurant", 3),
        ("revenge", 2), ("reverse", 2), ("rhythm", 2), ("riddle", 2),
        ("route", 1), ("rubber", 2), ("rule", 1), ("saddle", 2),
        ("sauce", 1), ("schedule", 2), ("science", 2), ("scourge", 1),
        ("sense", 1), ("separate", 3), ("several", 3), ("severe", 2),
        ("since", 1), ("single", 2), ("some", 1), ("sometimes", 2),
        ("somewhere", 2), ("source", 1), ("space", 1), ("special", 2),
        ("specific", 3), ("square", 1), ("squeeze", 1), ("stomach", 2),
        ("storage", 2), ("store", 1), ("strange", 1), ("structure", 2),
        ("struggle", 2), ("subtle", 2), ("suppose", 2), ("sure", 1),
        ("surface", 2), ("surprise", 2), ("table", 2), ("temperature", 4),
        ("terrible", 3), ("theme", 1), ("there", 1), ("these", 1),
        ("those", 1), ("thought", 1), ("through", 1), ("title", 2),
        ("together", 3), ("tongue", 1), ("torture", 2), ("total", 2),
        ("touched", 1), ("trouble", 2), ("turtle", 2), ("twelve", 1),
        ("type", 1), ("uncle", 2), ("unique", 2), ("universe", 3),
        ("used", 1), ("useful", 2), ("usual", 3), ("value", 2),
        ("variable", 4), ("vegetable", 4), ("vehicle", 3), ("village", 2),
        ("visible", 3), ("voice", 1), ("volume", 2), ("voyage", 2),
        ("vulnerable", 4), ("waste", 1), ("wednesday", 2), ("welcome", 2),
        ("were", 1), ("where", 1), ("while", 1), ("whistle", 2),
        ("whole", 1), ("whose", 1), ("women", 2), ("worse", 1),
        ("write", 1), ("wrote", 1),
    ];
    for &(word, count) in entries {
        m.insert(word, count);
    }
    m
});

pub fn lookup_exception(word: &str) -> Option<u8> {
    EXCEPTIONS.get(word).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_exceptions() {
        assert_eq!(lookup_exception("business"), Some(2));
        assert_eq!(lookup_exception("beautiful"), Some(3));
        assert_eq!(lookup_exception("colonel"), Some(2));
        assert_eq!(lookup_exception("queue"), Some(1));
        assert_eq!(lookup_exception("recipe"), Some(3));
    }

    #[test]
    fn unknown_word() {
        assert_eq!(lookup_exception("abcxyz"), None);
    }
}
