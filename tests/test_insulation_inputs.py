"""The insulation inputs reach `simulate` and the Python API through the one shared path (issue #919)."""
from ovd.simulate import simulate

LNG = dict(vessel="gas_barge", cargo="LNG", loa_m=140, beam_m=29, depth_m=12, freeboard_min_m=1.81,
           self_propelled=True, fill_ratio=0.98, max_tank_od_m=12.0, service_area="coastal-and-river")


def test_a_typed_conductivity_doubles_the_heat_leak_and_the_u_value():
    """Label: property
    """
    b, t = simulate(LNG), simulate(dict(LNG, tank_insulation_conductivity_w_per_mk=0.052))
    assert b["boil_off"]["evaluated"] and t["boil_off"]["evaluated"]
    assert abs(t["boil_off"]["heat_leak_kw"] / b["boil_off"]["heat_leak_kw"] - 2.0) < 1e-9
    assert abs(t["boil_off_detail"]["u_value_w_per_m2k"] / b["boil_off_detail"]["u_value_w_per_m2k"] - 2.0) < 1e-9
    assert t["boil_off"]["conductivity_w_per_mk"] == 0.052


def test_a_typed_density_scales_the_insulation_mass():
    """Label: property
    """
    base = simulate(LNG)["cost_masses"]["masses"]["insulation_t"]
    typed = simulate(dict(LNG, tank_insulation_density_kg_m3=2 * simulate(LNG)["drawing"]["tank"]["insulation_density_kg_m3"]))
    assert abs(typed["cost_masses"]["masses"]["insulation_t"] / base - 2.0) < 1e-9
