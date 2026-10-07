use crate::dice::DiceKind;
use crate::feature::Feature;

#[allow(dead_code)]
pub struct ClassDto {
    name: String,
    description: String,
    hit_dice: DiceKind,
}

#[allow(dead_code)]
pub struct ClassDefinition {
    name: String,
    description: String,
    hit_dice: DiceKind,
    class_levels: ClassLevels,
}

impl ClassDefinition {
    pub fn iter(&self) -> impl Iterator<Item = &ClassLevel> {
        self.class_levels.iter()
    }
}

pub struct ClassLevels {
    level1: ClassLevel,
    level2: ClassLevel,
    level3: ClassLevel,
    level4: ClassLevel,
    level5: ClassLevel,
    level6: ClassLevel,
    level7: ClassLevel,
    level8: ClassLevel,
    level9: ClassLevel,
    level10: ClassLevel,
    level11: ClassLevel,
    level12: ClassLevel,
    level13: ClassLevel,
    level14: ClassLevel,
    level15: ClassLevel,
    level16: ClassLevel,
    level17: ClassLevel,
    level18: ClassLevel,
    level19: ClassLevel,
    level20: ClassLevel,
}

impl ClassLevels {
    pub fn iter(&self) -> impl Iterator<Item = &ClassLevel> {
        ClassLevelsIterator::new(self)
    }
}

pub struct ClassLevelsIterator<'cl> {
    class_levels: &'cl ClassLevels,
    index: usize,
}

impl<'cl> Iterator for ClassLevelsIterator<'cl> {
    type Item = &'cl ClassLevel;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            1 => Some(&self.class_levels.level1),
            2 => Some(&self.class_levels.level2),
            3 => Some(&self.class_levels.level3),
            4 => Some(&self.class_levels.level4),
            5 => Some(&self.class_levels.level5),
            6 => Some(&self.class_levels.level6),
            7 => Some(&self.class_levels.level7),
            8 => Some(&self.class_levels.level8),
            9 => Some(&self.class_levels.level9),
            10 => Some(&self.class_levels.level10),
            11 => Some(&self.class_levels.level11),
            12 => Some(&self.class_levels.level12),
            13 => Some(&self.class_levels.level13),
            14 => Some(&self.class_levels.level14),
            15 => Some(&self.class_levels.level15),
            16 => Some(&self.class_levels.level16),
            17 => Some(&self.class_levels.level17),
            18 => Some(&self.class_levels.level18),
            19 => Some(&self.class_levels.level19),
            20 => Some(&self.class_levels.level20),
            _ => None
        }
    }
}

impl<'cl> ClassLevelsIterator<'cl> {
    fn new(class_levels: &'cl ClassLevels) -> Self {
        Self {
            class_levels,
            index: 0,
        }
    }
}

#[allow(dead_code)]
pub struct ClassLevel {
    features: Vec<Feature>,
}
