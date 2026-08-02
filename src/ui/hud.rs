use bevy::prelude::*;
use crate::ui::life_support::LifeSupport;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud_root)
            .add_systems(Update, update_life_support_hud.run_if(resource_exists_and_changed::<LifeSupport>));
    }
}

fn update_life_support_hud(
    life_support: Res<LifeSupport>,
    mut texts: ParamSet<(Query<&mut Text, With<OxygenText>>, Query<&mut Text, With<FoodText>>,
        Query<&mut Text, With<WaterText>>, Query<&mut Text, With<EnergyText>>,)>,
){
    let mut oxygen = texts.p0();
    let Ok(mut text) = oxygen.single_mut() else { return; };
    **text = format!("Oxygen: {:.0}", life_support.oxygen);

    let mut food = texts.p1();
    let Ok(mut text) = food.single_mut() else { return; };
    **text = format!("Food: {:.0}", life_support.food);

    let mut water = texts.p2();
    let Ok(mut text) = water.single_mut() else { return; };
    **text = format!("Water: {:.0}", life_support.water);

    let mut energy = texts.p3();
    let Ok(mut text) = energy.single_mut() else { return; };
    **text = format!("Energy: {:.0}", life_support.energy);
}

#[derive(Component)] pub struct OxygenText;
#[derive(Component)] pub struct FoodText;
#[derive(Component)] pub struct WaterText;
#[derive(Component)] pub struct EnergyText;

pub fn spawn_hud_root(mut commands: Commands) {
    commands.spawn((Node{
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
    ..Default::default()},)).with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                align_items: AlignItems::Center,
                padding: UiRect::horizontal(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        )).with_children(|bar| {
            bar.spawn((Text::new("Oxygen: 100"), OxygenText));
            bar.spawn((Text::new("Food: 100"), FoodText));
            bar.spawn((Text::new("Water: 100"), WaterText));
            bar.spawn((Text::new("Energy: 100"), EnergyText));

        });
    });
}


